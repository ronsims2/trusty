use std::io;
use std::io::Write;
use crate::sql::NoteSummary;

pub(crate) fn print_note_summary(note: NoteSummary) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let text = format!("{} {} {}", note.id, note.title, note.updated);
    writeln!(&mut handle, "{}", text).unwrap();
}