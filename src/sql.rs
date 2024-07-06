use std::process::exit;

use rusqlite::{Connection, named_params};
use uuid::Uuid;

use crate::cli::read_from_std_in;
use crate::errors::Errors;
use crate::render::{CrustyPrinter, print_note_summary, Printer};
use crate::security::{decrypt_dump, decrypt_note, encrypt_text, get_boss_key, prompt_for_password};
use crate::setup::{CrustyPathOperations, get_db_conn, PathOperations};
use crate::utils::{make_text_single_line, slice_text};

#[derive(Debug)]
pub struct NoteSummary {
    pub id: i32,
    pub title: String,
    pub updated: String,
}

pub struct SimpleNoteView {
    pub title: String,
    pub body: String,
    pub content_id: String,
    pub protected: bool
}

pub struct NoteView {
    pub title: String,
    pub body: String,
    pub note_id: i32,
    pub content_id: String,
    pub updated: String,
    pub created: String,
}

pub struct LargeNoteSummary {
    pub note_id: i32,
    pub title: String,
    pub content_id: String,
    pub content_size: i32
}

pub struct DBStats {
    pub total: i32,
    pub trashed: i32
}

pub struct SummaryStats {
    pub db_stats: DBStats,
    pub large_note_stats: LargeNoteSummary,
    pub state_note_stats: NoteView,
    pub fresh_note_stats: NoteView
}

pub struct KeyValuePair {
    pub key: String,
    pub value: String
}

pub fn add_note(cpo: &dyn PathOperations, title: &str, note: &str, protected: bool) -> bool {
    if protected {
        insert_encrypted_note(title, note);
    } else {
        let formatted_title = make_text_single_line(title);
        let truncated_title = slice_text(0, 128, &formatted_title);
        insert_note(cpo, &truncated_title, &note, false)
    }

    return true
}

pub(crate)  fn insert_note(cpo: &dyn PathOperations, title: &str, note: &str, protected: bool) {
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    // create the new note id
    let content_id = Uuid::new_v4().to_string();
    let note_insert = "INSERT INTO notes (title, protected, created, updated, content_id) \
    VALUES (:title, :protected, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, :content_id);";
    let content_insert = "INSERT INTO content (content_id, body) VALUES (:content_id, :body);";

    // The integrity of these 2 inserts needs to be guaranteed.
    conn.execute(&content_insert, named_params! {
        ":content_id": content_id,
        ":body": note,
    }).unwrap();

    conn.execute(&note_insert, named_params! {
        ":title": title,
        ":protected": protected,
        ":content_id": content_id,
    }).unwrap();

    let last_inserted_sql = "UPDATE app SET value = (SELECT last_insert_rowid()) WHERE key = 'last_touched';";
    conn.execute(last_inserted_sql, ()).unwrap();
}

pub(crate) fn insert_encrypted_note(title: &str, note: &str) {
    let encrypted_and_insert_note = | password: &str| -> bool {
        let formatted_title = make_text_single_line(title);
        let decrypted_boss_key = get_boss_key(password);
        let encrypted_title = encrypt_text(&decrypted_boss_key, &formatted_title);
        let encrypt_note = encrypt_text(&decrypted_boss_key, note);
        insert_note(&CrustyPathOperations{},&encrypted_title, &encrypt_note, true);

        return true
    };

    prompt_for_password(encrypted_and_insert_note, true, false);
}

pub fn list_note_titles(cpo: &dyn PathOperations, printer: &dyn Printer) {
    let sql = "SELECT note_id, title, updated, protected FROM notes WHERE TRASHED IS FALSE ORDER BY updated;";
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    let mut stmt = conn.prepare(sql).unwrap();
    let results = stmt.query_map([], |row| {
        let is_protected: bool = row.get(3).unwrap();
        let title: String = if is_protected { "🔒 ENCRYPTED".to_string() } else {row.get(1).unwrap_or("NULL".to_string())};
        Ok(NoteSummary {
            id: row.get(0)?,
            title,
            updated: row.get(2)?,
        })
    }).unwrap();

    for res in results {
        print_note_summary(printer, res.unwrap());
    }
}

pub fn get_note_by_id(cpo: &dyn PathOperations, id: usize) -> SimpleNoteView {
    let sql = "SELECT notes.title, content.body, notes.protected, notes.content_id FROM notes JOIN content on notes.content_id = content.content_id WHERE notes.note_id = :note_id;";
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    let mut stmt = conn.prepare(sql).unwrap();
    let result = match stmt.query_row(named_params! {":note_id": id as u32}, |row| {
        let is_protected = row.get(2).unwrap();

        let title: String = row.get(0).unwrap();
        let note: String = row.get(1).unwrap();

        return if is_protected {
            let unencrypted_note = decrypt_note(&title, &note);

            Ok(SimpleNoteView {
                title: unencrypted_note.title,
                body: unencrypted_note.body,
                content_id: row.get(3)?,
                protected: true
            })
        } else {
            Ok(SimpleNoteView {
                title,
                body: note,
                content_id: row.get(3)?,
                protected: false
            })
        }
    }) {
        Ok(res) => {
            update_last_touched(&CrustyPathOperations{},id.to_string().as_str());
            res
        },
        Err(_) => {
            CrustyPrinter{}.print_error(format!("Could not find note for id: {}", id));
            exit(Errors::NoteIdErr as i32);
        }
    };

    result
}

pub fn get_note_from_menu_line(cpo: &dyn PathOperations) -> SimpleNoteView {
    let result = match read_from_std_in() {
        None => {
            CrustyPrinter{}.print_error(format!("{}", "No menu line specified, could not lookup record."));
            exit(Errors::MenuLineErr as i32);
        }
        Some(ln) => {
            let trimmed_ln = ln.trim();
            if trimmed_ln.is_empty() {
                CrustyPrinter{}.print_error(format!("{}", "Menu line input is empty, could not lookup record."));
                exit(Errors::MenuLineEmptyErr as i32);
            } else {
                get_note_from_menu_line_by_id(cpo, ln.as_str())
            }
        }
    };
    result
}

pub fn get_note_from_menu_line_by_id(cpo: &dyn PathOperations, line: &str) -> SimpleNoteView {
    let mut id_segment = &line[0..9];
    id_segment = id_segment.trim();
    let result = match id_segment.parse::<i32>(){
        Ok(id) => {
            get_note_by_id(cpo, id as usize)
        }
        Err(_) => {
            CrustyPrinter{}.print_error(format!("{}", "Menu line input is malformed, please check your input."));
            exit(Errors::MenuLineMalformedErr as i32);
        }
    };
    result
}

pub fn update_last_touched(cpo: &dyn PathOperations, note_id:&str){
    let sql = "UPDATE app SET value = :last_touched WHERE key = 'last_touched';";
    match note_id.parse::<i32>() {
        Ok(id) => {
            let db_path = cpo.get_crusty_db_path();
            let conn = get_db_conn(&db_path);
            conn.execute(&sql, named_params! {":last_touched": id as usize}).unwrap();
        }
        Err(_) => {
            CrustyPrinter{}.print_error(format!("{}", "note ID is malformed, please check your input."));
            exit(Errors::NoteIdMalformedErr as i32)
        }
    }
}

pub fn get_last_touched_note(cpo: &dyn PathOperations) -> SimpleNoteView {
    let sql = "SELECT notes.content_id, notes.title, notes.protected, content.body FROM notes JOIN content on notes.content_id = content.content_id \
    WHERE notes.note_id = CAST((SELECT value FROM app WHERE key = 'last_touched') AS INTEGER); ";
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    let mut stmt = conn.prepare(sql).unwrap();
    let result = match stmt.query_row([], |row| {
        let is_protected: bool = row.get(2).unwrap();
        let title: String = row.get(1).unwrap();
        let body: String = row.get(3).unwrap();
        let content_id = row.get(0).unwrap();

        if is_protected {
            let decrypted_note = decrypt_note(&title, &body);

            Ok(SimpleNoteView {
                content_id,
                title: decrypted_note.title,
                body: decrypted_note.body,
                protected: is_protected
            })
        } else {
            Ok(SimpleNoteView {
                content_id,
                title: row.get(1)?,
                body: row.get(3)?,
                protected: is_protected
            })
        }
    }) {
        Ok(res) => {
            res
        }
        Err(_) => {
            CrustyPrinter{}.print_error(format!("{}", "Could not fetch the last touched note."));
            exit(Errors::LastTouchFetchErr as i32)
        }
    };
    result
}

pub fn update_note_ts_by_content_id(id: &str, conn: &Connection) -> bool {
    let sql = "UPDATE notes SET updated = CURRENT_TIMESTAMP WHERE content_id = :content_id;";
    let stmt = conn.prepare(sql);
    let result = stmt.unwrap().execute(named_params! {":content_id": id}).unwrap();

    result > 0
}

pub fn update_note_ts_by_note_id(id: usize, conn: &Connection) -> bool {
    let sql = "UPDATE notes SET updated = CURRENT_TIMESTAMP WHERE note_id = :note_id;";
    let stmt = conn.prepare(sql);
    let result = stmt.unwrap().execute(named_params! {":note_id": id}).unwrap();

    result > 0
}

pub fn update_note_by_content_id(cpo: &dyn PathOperations, id: &str, text: &str) -> bool {
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    let sql = "UPDATE content SET body = :body WHERE content_id = :content_id;";
    let stmt = conn.prepare(sql);
    stmt.unwrap().execute(named_params! {":content_id": id, ":body": &text}).unwrap();
    update_note_ts_by_content_id(id, &conn)
}

pub fn update_note_by_note_id(cpo: &dyn PathOperations, id: usize, text: &str) -> bool {
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    let sql = "UPDATE content SET body = :body WHERE content_id = (SELECT content_id FROM notes WHERE note_id = :note_id);";
    let stmt = conn.prepare(sql);
    let result = stmt.unwrap().execute(named_params! {":note_id": id, ":body": &text}).unwrap();
    update_note_ts_by_note_id(id, &conn);

    result > 0
}

pub fn update_title_by_content_id(cpo: &dyn PathOperations, id: &str, text: &str) -> bool {
    let title = make_text_single_line(&text);
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    let sql = "UPDATE notes SET title = :title, updated = CURRENT_TIMESTAMP WHERE content_id = :content_id;";
    let stmt = conn.prepare(sql);
    let result = stmt.unwrap().execute(named_params! {":content_id": id, ":title": &title}).unwrap();

    result > 0
}

pub fn delete_note_by_id(cpo: &dyn PathOperations, id: usize, force: bool) -> bool {
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    let sql = match force {
        true => {
            "DELETE FROM notes WHERE note_id = :note_id;"
        }
        false => {
            "DELETE FROM notes WHERE note_id = :note_id AND protected is FALSE;"
        }
    };
    let stmt = conn.prepare(sql);
    let result = stmt.unwrap().execute(named_params! {":note_id": id}).unwrap();

    result > 0
}

pub fn empty_trash(cpo: &dyn PathOperations) -> bool {
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    let sql = "DELETE FROM notes WHERE trashed is TRUE;";
    let result = conn.execute(&sql, ()).unwrap();

    result > 0
}

pub fn set_note_trash(cpo: &dyn PathOperations, id: usize, trash_state: bool) -> bool {
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    let sql = "UPDATE notes SET trashed = :trashed WHERE note_id = :note_id;";
    let stmt = conn.prepare(sql);
    let result = stmt.unwrap().execute(named_params! {":note_id": id, ":trashed": trash_state}).unwrap();

    result > 0
}

pub fn dump_notes(cpo: &dyn PathOperations, protected: bool) -> Vec<NoteView> {
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    let sql = "SELECT note_id, title, created, updated, notes.content_id, content.body from \
    notes JOIN content on notes.content_id = content.content_id WHERE protected is :protected;";
    let mut stmt = conn.prepare(sql).unwrap();

    let result_set = stmt.query_map(named_params! {":protected": protected},|row| {
        Ok(NoteView{
            note_id: row.get(0)?,
            title: row.get(1)?,
            created: row.get(2)?,
            updated: row.get(3)?,
            content_id: row.get(4)?,
            body: row.get(5)?,
        })
    }).unwrap();

    let mut results = vec![];
    for result in result_set {
        match result {
            Ok(res) => {
                results.push(res);
            }
            Err(_) => {
                results.push(NoteView{
                    title: "Error: could not fetch result".to_string(),
                    body: "".to_string(),
                    note_id: 0,
                    content_id: "".to_string(),
                    updated: "".to_string(),
                    created: "".to_string(),
                })
            }
        }
    }

    if protected {
        return decrypt_dump(&results)
    }

    results
}

pub fn get_summary(cpo: &dyn PathOperations) -> SummaryStats {
    let largest_note_sql = "SELECT note_id, title, content.content_id, \
    MAX(length(body)) from content JOIN notes on content.content_id = notes.content_id;";
    let stalest_note_sql = "SELECT note_id, title, content_id, MIN(updated) from notes;";
    let freshest_note_sql = "SELECT note_id, title, content_id, MAX(updated) from notes;";
    let total_trashed_sql = "SELECT (SELECT COUNT(note_id) from notes), \
    (SELECT COUNT(note_id) from notes WHERE trashed is TRUE);";

    let mut errors = vec![];

    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    let mut largest_stmt = conn.prepare(largest_note_sql).unwrap();
    let largest_result = match largest_stmt.query_row([], |row| {
        Ok(LargeNoteSummary{
            note_id: row.get(0)?,
            title: row.get(1)?,
            content_id: row.get(2)?,
            content_size: row.get(3)?,
        })
    }) {
        Ok(data) => {
            Some(data)
        }
        Err(_) => {
            errors.push("Error querying to determine largest note for summary.");
            None
        }
    };

    let mut stalest_stmt = conn.prepare(stalest_note_sql).unwrap();
    let stalest_result = match stalest_stmt.query_row([], |row| {
        Ok(NoteView{
            note_id: row.get(0)?,
            title: row.get(1)?,
            body: "".to_string(),
            content_id: row.get(2)?,
            updated: row.get(3)?,
            created: "".to_string(),
        })
    }) {
        Ok(data) => {
            Some(data)
        }
        Err(_) => {
            errors.push("Error querying oldest note for summary.");
            None
        }
    };

    let mut freshest_stmt = conn.prepare(freshest_note_sql).unwrap();
    let freshest_result = match freshest_stmt.query_row([], |row| {
        Ok(NoteView{
            note_id: row.get(0)?,
            title: row.get(1)?,
            body: "".to_string(),
            content_id: row.get(2)?,
            updated: row.get(3)?,
            created: "".to_string(),
        })
    }) {
        Ok(data) => {
            Some(data)
        }
        Err(_) => {
            errors.push("Error querying oldest note for summary.");
            None
        }
    };

    let mut total_trashed_stmt = conn.prepare(total_trashed_sql).unwrap();
    let total_trashed_result = match total_trashed_stmt.query_row([], |row| {
        Ok(DBStats{
            total: row.get(0)?,
            trashed: row.get(1)?
        })
    }) {
        Ok(data) => {
            Some(data)
        }
        Err(_) => {
            errors.push("Error querying stats.");
            None
        }
    };

    if errors.len() > 0 {
        CrustyPrinter{}.print_error("Error creating summary.".to_string());
        exit(Errors::SummaryErr as i32);
    }

    SummaryStats {
        db_stats: total_trashed_result.unwrap(),
        large_note_stats: largest_result.unwrap(),
        state_note_stats: stalest_result.unwrap(),
        fresh_note_stats: freshest_result.unwrap(),
    }

}

fn get_key_val_insert_sql(table: &str) -> String {
    format!("INSERT INTO {} (key, value) VALUES (:key, :value);", table)
}

fn get_key_val_select_sql(table: &str) -> String {
    format!("SELECT value from {} WHERE key = :key;", table)
}

fn get_key_val_update_sql(table: &str) -> String {
    format!("UPDATE {} SET value = :value WHERE key = :key;", table)
}

pub fn get_value_from_attr_table(cpo: &dyn PathOperations, table: &str, key: &str) -> KeyValuePair {
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    let sql = match table.to_lowercase().as_str() {
        "app" => {
            get_key_val_select_sql(table)
        },
        "config" => {
            get_key_val_select_sql(table)
        }
        _ => {
            CrustyPrinter{}.print_error(format!("{}", "Could not get select val sql."));
            exit(Errors::KeyValSelectErr as i32)
        }
    };

    let mut stmt = conn.prepare(&sql).unwrap();
    let result = match stmt.query_row(named_params! {":key": key},|row| {
        Ok(KeyValuePair {
            key: key.to_string(),
            value: row.get(0)?
        })
    }) {
        Ok(data) => {
            data
        },
        Err(_) => {
            CrustyPrinter{}.print_error(format!("{}", "Could not get select val sql."));
            exit(Errors::KeyValSelectErr as i32)
        }
    };
    result
}

pub fn add_key_value(cpo: &dyn PathOperations, table: &str, key: &str, value: &str) -> bool {
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);

    let sql = match table.to_lowercase().as_str() {
        // these match tables created during setup
        "app" => {
            get_key_val_insert_sql(table)
        },
        "config" => {
            get_key_val_insert_sql(table)
        }
        _ => {
            CrustyPrinter{}.print_error(format!("{}", "Could not create key val."));
            exit(Errors::KeyValInsertErr as i32)
        }
    };

    let stmt = conn.prepare(&sql);
    let code = stmt.unwrap().execute(named_params! {
        ":key": key,
        ":value" : value}).unwrap_or(0);

    code > 0
}

// @todo refactor to reuse/simplify key_val CRUD func logic
pub fn update_key_value(cpo: &dyn PathOperations, table: &str, key: &str, value: &str) -> bool {
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);

    let sql = match table.to_lowercase().as_str() {
        // these match tables created during setup
        // @todo these matches are probably over kill since this is trusted code being concatenated
        "app" => {
            get_key_val_update_sql(table)
        },
        "config" => {
            get_key_val_update_sql(table)
        }
        _ => {
            CrustyPrinter{}.print_error(format!("{}", "Could not update key val."));
            exit(Errors::KeyValUpdateErr as i32)
        }
    };

    // @todo this could be refactor into its own function
    let stmt = conn.prepare(&sql);
    let code = stmt.unwrap().execute(named_params! {
        ":key": key,
        ":value" : value}).unwrap_or(0);

    code > 0
}

pub fn update_protected_flag(cpo: &dyn PathOperations, note_id: usize, protected: bool) -> bool {
    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
   let sql = "UPDATE notes set protected = :protected WHERE note_id = :note_id;";

    let code = conn.execute(&sql, named_params! {
        ":note_id": note_id,
        ":protected": protected
    }).unwrap_or(0);

    code > 0
}

pub fn trash_note(cpo: &dyn PathOperations, id: usize) -> bool {
    set_note_trash(cpo, id, true)
}

pub fn restore_note(cpo: &dyn PathOperations, id: usize) -> bool {
    set_note_trash(cpo, id, false)
}

pub fn delete_note(cpo: &dyn PathOperations, note_id: usize, force: bool) -> bool {
    let result = delete_note_by_id(cpo, note_id, force);
    if result {
        CrustyPrinter{}.println(format!("Note: {} deleted.", note_id))
    } else {
        CrustyPrinter{}.println(format!("Could not delete noted: {}, it may be protected or already removed.", note_id));
    }
    return result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_key_val_insert_sql(){
        let sql = get_key_val_insert_sql("app");
        assert_eq!("INSERT INTO app (key, value) VALUES (:key, :value);", sql);
    }

    #[test]
    fn test_get_key_val_select_sql() {
        let sql = get_key_val_select_sql("app");
        assert_eq!("SELECT value from app WHERE key = :key;", sql);
    }

    #[test]
    fn test_get_key_val_update_sql() {
        let sql = get_key_val_update_sql("app");
        assert_eq!("UPDATE app SET value = :value WHERE key = :key;", sql)
    }


}