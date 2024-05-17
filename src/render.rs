use std::io;
use std::io::Write;
use crate::sql::{NoteSummary, SimpleNoteView};

pub(crate) fn print_note_summary(note: NoteSummary) {
    // @todo refactor to use cr_print
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let text = format!("{:width$} | {:title_width$} | {}", note.id, note.title, note.updated, width = 9, title_width = 45);
    writeln!(&mut handle, "{}", text).unwrap();
    writeln!(&mut handle, "{}", "-".repeat(80)).unwrap();
}

pub(crate) fn cr_println(text: String) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    writeln!(&mut handle, "{}", text).unwrap();
}

pub(crate) fn print_simple_note(note: SimpleNoteView) {
    // @todo Right now it is bette rto not render the title so that using things liked saved data are easier
    // cr_println(note.title);
    // cr_println(format!("{}", "_".repeat(80)));
    cr_println(note.body);
}

// @todo create print error