use std::fmt::format;
use std::io;
use std::io::Write;
use crate::sql::{NoteSummary, NoteView, SimpleNoteView, SummaryStats};
use crate::utils::{slice_text, truncate_rich_text};

#[cfg(test)]
use mockall::*;
#[cfg(test)]
use mockall::predicate::*;

pub(crate) fn print_note_summary(note: NoteSummary) {
    // @todo refactor to use cr_print
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    // let title = slice_text(0,45, &note.title);
    let title = truncate_rich_text(&note.title, 45);
    let text = format!("{:width$} | {} | {}", note.id, note.updated, title, width = 9);
    writeln!(&mut handle, "{}", text).unwrap();
    writeln!(&mut handle, "{}+{}+{}", "-".repeat(10), "-".repeat(21), "-".repeat(47)).unwrap();
}

#[cfg_attr(test, automock)]
pub trait Printer {
    fn cr_println(text: String);
    fn cr_print_error(text: String);
}

pub struct CrustyPrinter {}

impl Printer for CrustyPrinter {
    fn cr_println(text: String) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        writeln!(&mut handle, "{}", text).unwrap();
    }

    fn cr_print_error(text: String) {
        let stderr = io::stderr();
        let mut handle = stderr.lock();
        writeln!(&mut handle, "{}", text).unwrap();
    }
}

pub(crate) fn print_simple_note(note: SimpleNoteView) {
    // @todo Right now it is better to not render the title so that using things liked saved data are easier
    // cr_println(note.title);
    // cr_println(format!("{}", "_".repeat(80)));
    CrustyPrinter::cr_println(note.body);
}

// @todo create print error

pub(crate) fn print_dump(notes: Vec<NoteView>) {
    for note in notes {
        CrustyPrinter::cr_println(format!("{:width$} | {} | {} | {}", note.note_id, note.content_id, note.created, note.updated, width = 9));
        let lines = note.body.lines();
        for line in lines {
            CrustyPrinter::cr_println(format!("{:width$} | {} | {}", note.note_id, note.content_id, line, width = 9));
        }
    }
}

pub(crate) fn print_app_summary(summary: SummaryStats) {
    let total = summary.db_stats.total;
    let trashed = summary.db_stats.trashed;
    let stale = summary.state_note_stats;
    let fresh = summary.fresh_note_stats;
    let largest_note = summary.large_note_stats;
    CrustyPrinter::cr_println(format!("{}", "cRusty 🦀📝 Summary"));
    CrustyPrinter::cr_println(format!("{}", "=".repeat(80)));

    CrustyPrinter::cr_println(format!("Total Notes: {} :: Trashed Notes: {}", total, trashed));
    CrustyPrinter::cr_println(format!("{}", "=".repeat(80)));

    CrustyPrinter::cr_println(format!("{}", "Largest Note:"));
    CrustyPrinter::cr_println(format!("Note ID: {:width$}", largest_note.note_id, width = 9));
    CrustyPrinter::cr_println(format!("Content ID: {}", largest_note.content_id));
    CrustyPrinter::cr_println(format!("Notes Size: {} (chars)", largest_note.content_size));
    CrustyPrinter::cr_println(format!("Title: {}", largest_note.title));
    CrustyPrinter::cr_println(format!("{}", "=".repeat(80)));

    CrustyPrinter::cr_println(format!("{}", "Freshest Note"));
    CrustyPrinter::cr_println(format!("Note ID   | Content ID                           | Updated           "));
    CrustyPrinter::cr_println(format!("{:width$} | {} | {}",
                       fresh.note_id,
                       fresh.content_id,
                       fresh.updated,
                       width = 9));
    CrustyPrinter::cr_println(format!("Title: {}", fresh.title));
    CrustyPrinter::cr_println(format!("{}", "=".repeat(80)));
    CrustyPrinter::cr_println(format!("Note ID   | Content ID                           | Updated           "));
    CrustyPrinter::cr_println(format!("{:width$} | {} | {}",
                       stale.note_id,
                       stale.content_id,
                       stale.updated,
                       width = 9));
    CrustyPrinter::cr_println(format!("Title: {}", stale.title));
    CrustyPrinter::cr_println(format!("{}", "=".repeat(80)));
}

#[cfg(test)]
mod tests {
    use crate::render::print_note_summary;
    use crate::sql::NoteSummary;

    #[test]
    fn test_print_note_summary() {
        let test_note_summary = NoteSummary{
            id: 1,
            title: "Get Started with cRusty".to_string(),
            updated: "2024-07-01 22:56:27".to_string(),
        };

        let sys_new_line = "";

        let control_output = format!("        1 | 2024-07-01 22:56:27 | Get Started with cRusty{}
        ----------+---------------------+-----------------------------------------------", sys_new_line);

        print_note_summary(test_note_summary);

    }
}