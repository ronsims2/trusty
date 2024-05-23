use std::convert::Infallible;
use std::fmt::format;
use std::num::ParseIntError;
use std::process::exit;
use rusqlite::named_params;
use uuid::Uuid;
use crate::cli::read_from_std_in;
use crate::render::{cr_println, print_note_summary};
use crate::setup::get_crusty_db_conn;

#[derive(Debug)]
pub(crate) struct NoteSummary {
    pub(crate) id: i32,
    pub(crate) title: String,
    pub(crate) updated: String,
}

pub(crate)struct SimpleNoteView {
    pub(crate) title: String,
    pub(crate) body: String,
}

pub(crate)struct UpdateNoteView {
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) id: String,
}

pub(crate) fn insert_note(title: &str, note: &str, protected: bool) {
    let conn = get_crusty_db_conn();
    let protected_val = if protected {1} else {0};
    // create the new note id
    let content_id = Uuid::new_v4().to_string();
    let note_insert = "INSERT INTO notes (title, protected, created, updated, content_id) \
    VALUES (:title, :protected, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, :content_id);";
    let content_insert = "INSERT INTO content (content_id, body) VALUES (:content_id, :body);";


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

pub(crate) fn list_note_titles() {
    let sql = "SELECT note_id, title, updated FROM notes WHERE protected = FALSE ORDER BY updated;";
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

pub(crate) fn update_note_by_content_id(id: &str, text: &str) {
    let conn = get_crusty_db_conn();
    let sql = "UPDATE content SET body = :body WHERE content_id = :content_id;";
    let stmt = conn.prepare(sql);
    stmt.unwrap().execute(named_params! {":content_id": id, ":body": &text}).unwrap();
}

pub(crate) fn update_note_by_note_id(id: usize, text: &str) {
    let conn = get_crusty_db_conn();
    let sql = "UPDATE content SET body = :body WHERE content_id = (SELECT content_id FROM notes WHERE note_id = :note_id);";
    let stmt = conn.prepare(sql);
    stmt.unwrap().execute(named_params! {":note_id": id, ":body": &text}).unwrap();
}