use std::io;
use std::io::Write;
use crate::sql::NoteSummary;

pub(crate) fn print_note_summary(note: NoteSummary) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let text = format!("{:width$} | {:title_width$} | {}", note.id, note.title, note.updated, width = 9, title_width = 45);
    writeln!(&mut handle, "{}", text).unwrap();
    writeln!(&mut handle, "{}", "-".repeat(80)).unwrap();
}