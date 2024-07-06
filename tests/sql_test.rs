/**
* Add test here that need to check the db
*/
use std::path::PathBuf;
use mockall::automock;
use tempfile::tempdir;
use crusty::setup::{create_crusty_dir, get_db_conn, init_crusty_db, PathOperations};
use crusty::sql::{get_note_by_id, add_note, list_note_titles, get_note_from_menu_line_by_id, NoteSummary, SimpleNoteView, update_last_touched, get_last_touched_note, update_note_ts_by_note_id, update_note_ts_by_content_id, update_note_by_note_id, update_note_by_content_id, update_title_by_content_id, delete_note, get_summary, set_note_trash, empty_trash, dump_notes, get_value_from_attr_table, add_key_value, update_key_value, update_protected_flag, trash_note, restore_note};
use crusty::render::Printer;


struct TestPrinter{}
#[cfg_attr(test, automock)]
impl Printer for TestPrinter {fn println(&self, text: String) -> () {
        todo!()
    }

fn print_error(&self, text: String) -> () {
        todo!()
    }

}

pub struct TestPathOperations {
    cached_path: PathBuf
}

impl PathOperations for TestPathOperations {
    fn get_crusty_dir(&self) -> PathBuf {
        self.cached_path.to_path_buf()
    }
    fn get_crusty_db_path(&self) -> PathBuf {
        self.get_crusty_dir().join("crusty.db")
    }
}

pub fn create_test_db<F>(mut test_fun: F) where F: FnMut(&dyn PathOperations) {
    let mut fake = TestPathOperations{ cached_path: tempdir().unwrap().into_path().join(".crusty")};


    create_crusty_dir(&fake);
    init_crusty_db(&fake);

    // Call test
    test_fun(&fake)
}


fn test_default_note(note: SimpleNoteView) {
    assert!(note.title.eq("Get Started with cRusty"));
    assert!(note.body.eq("Welcome to cRusty the CLI notes app. -Ron"));
}


#[test]
fn test_add_note() {
    let test = | mock: &dyn PathOperations | {
        let title = "foo";
        let body = "bar";
        add_note(mock, title, body, false);
        let note = get_note_by_id(mock, 2);
        assert_eq!(note.title, title);
        assert_eq!(note.body, body);
        // todo! only test protected flows using E2E so that prompts are not opened
    };

    create_test_db(test);
}

// Test encrypted paths using python E2E tests
#[test]
fn test_populate_crusty_sys_tables() {
    // this also test get_note_by_id
    let test = | mock: &dyn PathOperations | {
        let note = get_note_by_id(mock, 1);
        // This tests the setup of the notes and content table from a users perspective
        // this test both create_crusty_sys_tables and populate_crusty_sys_tables
        test_default_note(note);
    };

    create_test_db(test);
}

#[test]
fn test_list_note_titles() {
    let test = | mock: &dyn PathOperations | {
        let mut mock_printer = MockTestPrinter::new();
        mock_printer.expect_println().times(2).return_const(());
        mock_printer.expect_print_error().times(0).return_const(());

        list_note_titles(mock, &mock_printer);
    };

    create_test_db(test);
}

#[test]
fn test_get_note_from_menu_line_by_id() {
    let test = | mock: &dyn PathOperations | {
        let test_line = "        1 | 2024-07-01 22:56:27 | Get Started with cRusty";
        let note = get_note_from_menu_line_by_id(mock, test_line);
        test_default_note(note);
    };

    create_test_db(test);
}

#[test]
fn test_update_last_touched() {
    let test = | mock: &dyn PathOperations | {
        let title = "foo";
        add_note(mock, title, "bar", false);
        let note_1 = get_last_touched_note(mock);
        assert_eq!(note_1.title, title);
        update_last_touched(mock, "1");
        let note_2 = get_last_touched_note(mock);
        assert_ne!(note_2.title, title);
    };

    create_test_db(test);
}

#[test]
fn test_update_note_functions() {
    let test = | mock: &dyn PathOperations | {
        let conn = get_db_conn(&mock.get_crusty_db_path());
        let note = get_note_by_id(mock, 1);
        assert!(update_note_ts_by_note_id(1, &conn));
        assert!(update_note_ts_by_content_id(&note.content_id, &conn));
        let text = "foobar";
        assert!(update_note_by_note_id(mock, 1, text));
        let note_2 = get_note_by_id(mock, 1);
        assert_eq!(note_2.body, text);
        let text_2 = "barbaz";
        assert!(update_note_by_content_id(mock, &note_2.content_id, text_2));
        let note_3 = get_note_by_id(mock, 1);
        assert_eq!(note_3.body, text_2);
        let text_3 = "foo title";
        assert!(update_title_by_content_id(mock, &note_3.content_id, text_3));
        let note_4 = get_note_by_id(mock, 1);
        assert_eq!(note_4.title, text_3);
    };

    create_test_db(test);
}

// this also tests get summary
#[test]
fn test_delete_note() {
    let test = | mock: &dyn PathOperations | {
        assert!(delete_note(mock, 1, true));
        // db needs a note to get summary right now
        add_note(mock, "foo", "bar", false);
        let summary = get_summary(mock);
        assert_eq!(summary.db_stats.total, 1);
    };

    create_test_db(test);
}

#[test]
fn test_trash_feature() {
    let test = | mock: &dyn PathOperations | {
        assert!(set_note_trash(mock, 1, true));
        let summary = get_summary(mock);
        assert_eq!(summary.db_stats.trashed, 1);
        empty_trash(mock);
        // db needs a note to get summary right now
        add_note(mock, "foo", "bar", false);
        let summary_2 = get_summary(mock);
        assert_eq!(summary_2.db_stats.trashed, 0);
        // test trash wrapper funcs
        add_note(mock, "foo", "bar", false);
        assert!(trash_note(mock, 3));
        assert!(restore_note(mock, 3));
        assert!(delete_note(mock, 3, false));
    };

    create_test_db(test);

}

#[test]
fn test_dump_notes() {
    let test = | mock: &dyn PathOperations | {
        add_note(mock, "foo", "bar", false);
        add_note(mock, "foo", "bar", false);
        add_note(mock, "foo", "bar", false);
        let notes = dump_notes(mock, false);
        assert_eq!(notes.len(), 4);
    };

    create_test_db(test);
}

#[test]
fn test_attr_functions() {
    let test = | mock: &dyn PathOperations | {
        let value = get_value_from_attr_table(mock, "app", "last_touched");
        assert_eq!(value.value, "0");
        let key_2 = "foo";
        let val_2 = "42";
        add_key_value(mock, "app", key_2, val_2);
        let value_2 = get_value_from_attr_table(mock, "app", key_2);
        assert_eq!(value_2.value, val_2);
        let updated_val = "99";
        update_key_value(mock, "app", key_2, updated_val);
        let updated_value = get_value_from_attr_table(mock, "app", key_2);
        assert_eq!(updated_value.value, updated_val);
    };

    create_test_db(test);
}

#[test]
fn test_update_protected_flag() {
    let test = | mock: &dyn PathOperations | {
        assert!(update_protected_flag(mock, 1, true));
    };

    create_test_db(test);
}
