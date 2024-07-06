use std::io;
use std::io::Write;

#[cfg(test)]
use mockall::*;

use crate::sql::{NoteSummary, NoteView, SimpleNoteView, SummaryStats};
use crate::utils::truncate_rich_text;

pub(crate) fn print_note_summary(printer: &dyn Printer, note: NoteSummary) {
    let title = truncate_rich_text(&note.title, 45);
    let text = format!("{:width$} | {} | {}", note.id, note.updated, title, width = 9);
    printer.println(text);
    printer.println(format!("{}+{}+{}", "-".repeat(10), "-".repeat(21), "-".repeat(47)));
}

//#[cfg_attr(test, automock)]
#[cfg_attr(test, automock)]
pub trait Printer {
    fn println(&self, text: String) -> ();
    fn print_error(&self, text: String) -> ();
}

pub struct CrustyPrinter {}

impl Printer for CrustyPrinter {
    fn println(&self, text: String) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        writeln!(&mut handle, "{}", text).unwrap();
    }

    fn print_error(&self, text: String) {
        let stderr = io::stderr();
        let mut handle = stderr.lock();
        writeln!(&mut handle, "{}", text).unwrap();
    }
}

pub(crate) fn print_simple_note(printer: &dyn Printer, note: SimpleNoteView) {
    // @todo Right now it is better to not render the title so that using things liked saved data are easier
    // cr_println(note.title);
    // cr_println(format!("{}", "_".repeat(80)));
    printer.println(note.body);
}

pub(crate) fn print_dump(printer: &dyn Printer, notes: Vec<NoteView>) {
    for note in notes {
        printer.println(format!("{:width$} | {} | {} | {}", note.note_id, note.content_id, note.created, note.updated, width = 9));
        let lines = note.body.lines();
        for line in lines {
            printer.println(format!("{:width$} | {} | {}", note.note_id, note.content_id, line, width = 9));
        }
    }
}

pub(crate) fn print_app_summary(printer: &dyn Printer, summary: SummaryStats) {
    let total = summary.db_stats.total;
    let trashed = summary.db_stats.trashed;
    let stale = summary.state_note_stats;
    let fresh = summary.fresh_note_stats;
    let largest_note = summary.large_note_stats;
    printer.println(format!("{}", "cRusty 🦀📝 Summary"));
    printer.println(format!("{}", "=".repeat(80)));

    printer.println(format!("Total Notes: {} :: Trashed Notes: {}", total, trashed));
    printer.println(format!("{}", "=".repeat(80)));

    printer.println(format!("{}", "Largest Note:"));
    printer.println(format!("Note ID: {:width$}", largest_note.note_id, width = 9));
    printer.println(format!("Content ID: {}", largest_note.content_id));
    printer.println(format!("Notes Size: {} (chars)", largest_note.content_size));
    printer.println(format!("Title: {}", largest_note.title));
    printer.println(format!("{}", "=".repeat(80)));

    printer.println(format!("{}", "Freshest Note"));
    printer.println(format!("Note ID   | Content ID                           | Updated           "));
    printer.println(format!("{:width$} | {} | {}",
                                   fresh.note_id,
                                   fresh.content_id,
                                   fresh.updated,
                                   width = 9));
    printer.println(format!("Title: {}", fresh.title));
    printer.println(format!("{}", "=".repeat(80)));
    printer.println(format!("Note ID   | Content ID                           | Updated           "));
    printer.println(format!("{:width$} | {} | {}",
                                   stale.note_id,
                                   stale.content_id,
                                   stale.updated,
                                   width = 9));
    printer.println(format!("Title: {}", stale.title));
    printer.println(format!("{}", "=".repeat(80)));
}

#[cfg(test)]
mod tests {
    use crate::sql::{DBStats, LargeNoteSummary};

    use super::*;

    #[test]
    fn test_print_note_summary() {
        let test_note_summary = NoteSummary{
            id: 1,
            title: "Get Started with cRusty".to_string(),
            updated: "2024-07-01 22:56:27".to_string(),
        };

        let mut mock = MockPrinter::new();
        mock.expect_println().times(2).return_const(());
        mock.expect_print_error().times(0).return_const(());

        print_note_summary(&mock, test_note_summary);
    }
    
    #[test]
    fn test_print_simple_note() {
        let test_note = SimpleNoteView{
            title: "".to_string(),
            body: "".to_string(),
            content_id: "".to_string(),
            protected: false,
        };

        let mut mock = MockPrinter::new();
        mock.expect_println().times(1).return_const(());
        mock.expect_print_error().times(0).return_const(());
        print_simple_note(&mock, test_note);
    }

    #[test]
    fn test_print_dump() {
        let mock_data = vec![NoteView{
            title: "foo".to_string(),
            body: "foofoo".to_string(),
            note_id: 0,
            content_id: "".to_string(),
            updated: "".to_string(),
            created: "".to_string(),
        }, NoteView{
            title: "bar".to_string(),
            body: "bar\r\nbar".to_string(),
            note_id: 0,
            content_id: "".to_string(),
            updated: "".to_string(),
            created: "".to_string(),
        }];

        let mut mock = MockPrinter::new();
        mock.expect_println().times(5).return_const(());
        mock.expect_print_error().times(0).return_const(());

        print_dump(&mock, mock_data);
    }

    #[test]
    fn test_print_app_summary() {
        let mock_data = SummaryStats{
            db_stats: DBStats { total: 0, trashed: 0 },
            large_note_stats: LargeNoteSummary {
                note_id: 0,
                title: "".to_string(),
                content_id: "".to_string(),
                content_size: 0,
            },
            state_note_stats: NoteView {
                title: "".to_string(),
                body: "".to_string(),
                note_id: 0,
                content_id: "".to_string(),
                updated: "".to_string(),
                created: "".to_string(),
            },
            fresh_note_stats: NoteView {
                title: "".to_string(),
                body: "".to_string(),
                note_id: 0,
                content_id: "".to_string(),
                updated: "".to_string(),
                created: "".to_string(),
            },
        };

        let mut mock = MockPrinter::new();
        mock.expect_println().times(19).return_const(());
        mock.expect_print_error().times(0).return_const(());

        print_app_summary(&mock, mock_data);
    }
}