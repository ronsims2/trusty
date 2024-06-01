use std::convert::Infallible;
use std::fmt::format;
use std::num::ParseIntError;
use std::process::exit;
use rusqlite::{Connection, MappedRows, named_params, Row};
use uuid::Uuid;
use crate::cli::read_from_std_in;
use crate::render::{cr_println, print_note_summary};
use crate::setup::get_crusty_db_conn;
use crate::utils::make_text_single_line;

#[derive(Debug)]
pub(crate) struct NoteSummary {
    pub(crate) id: i32,
    pub(crate) title: String,
    pub(crate) updated: String,
}

pub(crate) struct SimpleNoteView {
    pub(crate) title: String,
    pub(crate) body: String,
}

pub(crate) struct NoteView {
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) note_id: i32,
    pub(crate) content_id: String,
    pub(crate) updated: String,
    pub(crate) created: String,
}

pub(crate) struct UpdateNoteView {
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) id: String,
}

pub(crate) struct LargeNoteSummary {
    pub note_id: i32,
    pub title: String,
    pub content_id: String,
    pub content_size: i32
}

pub(crate) struct DBStats {
    pub total: i32,
    pub trashed: i32
}

pub(crate) struct SummaryStats {
    pub db_stats: DBStats,
    pub large_note_stats: LargeNoteSummary,
    pub state_note_stats: NoteView,
    pub fresh_note_stats: NoteView
}

pub(crate) fn insert_note(title: &str, note: &str, protected: bool) {
    let formatted_title = make_text_single_line(title);
    let conn = get_crusty_db_conn();
    let protected_val = if protected {1} else {0};
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
        ":title": formatted_title,
        ":protected": protected,
        ":content_id": content_id,
    }).unwrap();

    let last_inserted_sql = "UPDATE app SET value = (SELECT last_insert_rowid()) WHERE key = 'last_touched';";
    conn.execute(last_inserted_sql, ()).unwrap();
}

pub(crate) fn list_note_titles() {
    let sql = "SELECT note_id, title, updated FROM notes WHERE protected = FALSE AND TRASHED IS FALSE ORDER BY updated;";
    let conn = get_crusty_db_conn();
    let mut stmt = conn.prepare(sql).unwrap();
    let results = stmt.query_map([], |row| {
        Ok(NoteSummary {
            id: row.get(0)?,
            title: row.get(1)?,
            updated: row.get(2)?,
        })
    }).unwrap();

    for res in results {
        print_note_summary(res.unwrap());
    }
}

pub(crate) fn get_note_by_id(id: usize) -> SimpleNoteView {
    let sql = "SELECT notes.title, content.body FROM notes JOIN content on notes.content_id = content.content_id WHERE notes.note_id = :note_id;";
    let conn = get_crusty_db_conn();
    let mut stmt = conn.prepare(sql).unwrap();
    let result = match stmt.query_row(named_params! {":note_id": id as u32}, |row| {
        Ok(SimpleNoteView {
            title: row.get(0)?,
            body: row.get(1)?,
        })
    }) {
        Ok(res) => {
            update_last_touched(id.to_string().as_str());
            res
        },
        Err(err) => {
            cr_println(format!("Could not find note for id: {}", id));
            exit(504);
        }
    };

    result
}

pub(crate) fn get_note_from_menu_line() -> SimpleNoteView {
    let result = match read_from_std_in() {
        None => {
            cr_println(format!("{}", "No menu line specified, could not lookup record."));
            exit(506);
        }
        Some(ln) => {
            let trimmed_ln = ln.trim();
            if trimmed_ln.is_empty() {
                cr_println(format!("{}", "Menu line input is empty, could not lookup record."));
                exit(507);
            } else {
                get_note_from_menu_line_by_id(ln.as_str())
            }
        }
    };
    result
}

fn get_note_from_menu_line_by_id(line: &str) -> SimpleNoteView {
    let mut id_segment = &line[0..9];
    id_segment = id_segment.trim();
    let result = match id_segment.parse::<i32>(){
        Ok(id) => {
            get_note_by_id(id as usize)
        }
        Err(_) => {
            cr_println(format!("{}", "Menu line input is malformed, please check your input."));
            exit(507);
        }
    };
    result
}

pub(crate) fn update_last_touched(note_id:&str){
    let sql = "UPDATE app SET value = :last_touched WHERE key = 'last_touched';";
    match note_id.parse::<i32>() {
        Ok(id) => {
            let conn = get_crusty_db_conn();
            conn.execute(&sql, named_params! {":last_touched": id as usize}).unwrap();
        }
        Err(_) => {
            cr_println(format!("{}", "note ID is malformed, please check your input."));
            exit(508)
        }
    }
}

pub(crate) fn get_last_touched_note() -> UpdateNoteView {
    let sql = "SELECT notes.content_id, notes.title, content.body FROM notes JOIN content on notes.content_id = content.content_id \
    WHERE notes.note_id = CAST((SELECT value FROM app WHERE key = 'last_touched') AS INTEGER); ";
    let conn = get_crusty_db_conn();
    let mut stmt = conn.prepare(sql).unwrap();
    let result = match stmt.query_row([], |row| {
        Ok(UpdateNoteView {
            id: row.get(0)?,
            title: row.get(1)?,
            body: row.get(2)?,
        })
    }) {
        Ok(res) => {
            res
        }
        Err(_) => {
            cr_println(format!("{}", "Could not fetch the last touched note."));
            exit(509)
        }
    };
    result
}

pub(crate) fn update_note_ts_by_content_id(id: &str, conn: &Connection) {
    let sql = "UPDATE notes SET updated = CURRENT_TIMESTAMP WHERE content_id = :content_id;";
    let stmt = conn.prepare(sql);
    stmt.unwrap().execute(named_params! {":content_id": id}).unwrap();
}

pub(crate) fn update_note_ts_by_note_id(id: usize, conn: &Connection) {
    let sql = "UPDATE notes SET updated = CURRENT_TIMESTAMP WHERE note_id = :note_id;";
    let stmt = conn.prepare(sql);
    stmt.unwrap().execute(named_params! {":note_id": id}).unwrap();
}

pub(crate) fn update_note_by_content_id(id: &str, text: &str) {
    let conn = get_crusty_db_conn();
    let sql = "UPDATE content SET body = :body WHERE content_id = :content_id;";
    let stmt = conn.prepare(sql);
    stmt.unwrap().execute(named_params! {":content_id": id, ":body": &text}).unwrap();
    update_note_ts_by_content_id(id, &conn)
}

pub(crate) fn update_note_by_note_id(id: usize, text: &str) {
    let conn = get_crusty_db_conn();
    let sql = "UPDATE content SET body = :body WHERE content_id = (SELECT content_id FROM notes WHERE note_id = :note_id);";
    let stmt = conn.prepare(sql);
    stmt.unwrap().execute(named_params! {":note_id": id, ":body": &text}).unwrap();
    update_note_ts_by_note_id(id, &conn)
}

pub(crate) fn update_title_by_content_id(id: &str, text: &str) {
    let title = make_text_single_line(&text);
    let conn = get_crusty_db_conn();
    let sql = "UPDATE notes SET title = :title, updated = CURRENT_TIMESTAMP WHERE content_id = :content_id;";
    let stmt = conn.prepare(sql);
    stmt.unwrap().execute(named_params! {":content_id": id, ":title": &title}).unwrap();
}

pub(crate) fn delete_note(id: usize, force: bool) {
    let conn = get_crusty_db_conn();
    let sql = match force {
        true => {
            "DELETE FROM notes WHERE note_id = :note_id;"
        }
        false => {
            "DELETE FROM notes WHERE note_id = :note_id AND protect is FALSE;"
        }
    };
    let stmt = conn.prepare(sql);
    stmt.unwrap().execute(named_params! {":note_id": id}).unwrap();
}

pub(crate) fn empty_trash() {
    let conn = get_crusty_db_conn();
    let sql = "DELETE FROM notes WHERE trashed is TRUE;";
    conn.execute(&sql, ()).unwrap();
}

pub(crate) fn set_note_trash(id: usize, trash_state: bool) {
    let conn = get_crusty_db_conn();
    let sql = "UPDATE notes SET trashed = :trashed WHERE note_id = :note_id;";
    let stmt = conn.prepare(sql);
    stmt.unwrap().execute(named_params! {":note_id": id}).unwrap();
}

pub(crate) fn dump_notes() -> Vec<NoteView> {
    let conn = get_crusty_db_conn();
    let sql = "SELECT note_id, title, created, updated, notes.content_id, content.body from \
    notes JOIN content on notes.content_id = content.content_id WHERE protected is FALSE AND trashed IS FALSE;";
    let mut stmt = conn.prepare(sql).unwrap();

    let result_set = stmt.query_map([],|row| {
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

    results
}

pub(crate) fn get_summary() -> SummaryStats {
    let largest_note_sql = "SELECT note_id, title, content.content_id, \
    MAX(length(body)) from content JOIN notes on content.content_id = notes.content_id;";
    let stalest_note_sql = "SELECT note_id, title, content_id, MIN(updated) from notes;";
    let freshest_note_sql = "SELECT note_id, title, content_id, MAX(updated) from notes;";
    let total_trashed_sql = "SELECT (SELECT COUNT(note_id) from notes), \
    (SELECT COUNT(note_id) from notes WHERE trashed is TRUE);";

    let mut errors = vec![];

    let conn = get_crusty_db_conn();
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
        Err(err) => {
            errors.push("Error querying to determine largest note for summary.");
            None
        }
    };

    let mut stalest_stmt = conn.prepare(stalest_note_sql).unwrap();
    let stalest_result = match largest_stmt.query_row([], |row| {
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
        Err(error) => {
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
        Err(error) => {
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
        Err(error) => {
            errors.push("Error querying stats.");
            None
        }
    };

    SummaryStats {
        db_stats: total_trashed_result.unwrap(),
        large_note_stats: largest_result.unwrap(),
        state_note_stats: stalest_result.unwrap(),
        fresh_note_stats: freshest_result.unwrap(),
    }

}