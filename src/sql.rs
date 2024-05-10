use std::fmt::format;
use rusqlite::named_params;
use uuid::Uuid;
use crate::setup::get_crusty_db_conn;

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