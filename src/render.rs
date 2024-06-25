use std::fmt::format;
use std::io;
use std::io::Write;
use crate::sql::{NoteSummary, NoteView, SimpleNoteView, SummaryStats};
use crate::utils::{slice_text, truncate_rich_text};

pub(crate) fn print_note_summary(note: NoteSummary) {
    // @todo refactor to use cr_print
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    // let title = slice_text(0,45, &note.title);
    let title = truncate_rich_text(&note.title, 45);
    let text = format!("{:width$} | {} | {}", note.id, note.updated, title, width = 9);
    writeln!(&mut handle, "{}", text).unwrap();
    writeln!(&mut handle, "{}+{}+{}", "-".repeat(10), "-".repeat(47), "-".repeat(21)).unwrap();
}

pub(crate) fn cr_println(text: String) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    writeln!(&mut handle, "{}", text).unwrap();
}

pub(crate) fn print_simple_note(note: SimpleNoteView) {
    // @todo Right now it is better to not render the title so that using things liked saved data are easier
    // cr_println(note.title);
    // cr_println(format!("{}", "_".repeat(80)));
    cr_println(note.body);
}

// @todo create print error

pub(crate) fn print_dump(notes: Vec<NoteView>) {
    for note in notes {
        cr_println(format!("{:width$} | {} | {} | {}", note.note_id, note.content_id, note.created, note.updated, width = 9));
        let lines = note.body.lines();
        for line in lines {
            cr_println(format!("{:width$} | {} | {}", note.note_id, note.content_id, line, width = 9));
        }
    }
}

pub(crate) fn print_app_summary(summary: SummaryStats) {
    let total = summary.db_stats.total;
    let trashed = summary.db_stats.trashed;
    let stale = summary.state_note_stats;
    let fresh = summary.fresh_note_stats;
    let largest_note = summary.large_note_stats;
    cr_println(format!("{}", "cRusty 🦀📝 Summary"));
    cr_println(format!("{}", "=".repeat(80)));

    cr_println(format!("Total Notes: {} :: Trashed Notes: {}", total, trashed));
    cr_println(format!("{}", "=".repeat(80)));

    cr_println(format!("{}", "Largest Note:"));
    cr_println(format!("Note ID: {:width$}", largest_note.note_id, width = 9));
    cr_println(format!("Content ID: {}", largest_note.content_id));
    cr_println(format!("Notes Size: {} (chars)", largest_note.content_size));
    cr_println(format!("Title: {}", largest_note.title));
    cr_println(format!("{}", "=".repeat(80)));

    cr_println(format!("{}", "Freshest Note"));
    cr_println(format!("Note ID   | Content ID                           | Updated           "));
    cr_println(format!("{:width$} | {} | {}",
                       fresh.note_id,
                       fresh.content_id,
                       fresh.updated,
                       width = 9));
    cr_println(format!("Title: {}", fresh.title));
    cr_println(format!("{}", "=".repeat(80)));
    cr_println(format!("Note ID   | Content ID                           | Updated           "));
    cr_println(format!("{:width$} | {} | {}",
                       stale.note_id,
                       stale.content_id,
                       stale.updated,
                       width = 9));
    cr_println(format!("Title: {}", stale.title));
    cr_println(format!("{}", "=".repeat(80)));
}