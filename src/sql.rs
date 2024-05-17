use std::fmt::format;
use std::process::exit;
use rusqlite::named_params;
use uuid::Uuid;
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
        Ok(res) => res,
        Err(err) => {
            cr_println(format!("Could not find note for id: {}", id));
            exit(504);
        }
    };

    result
}