/**
* Add test here that need to check the db
*/
use std::path::PathBuf;
use mockall::automock;
use tempfile::tempdir;
use crusty::setup::{create_crusty_dir, init_crusty_db, PathOperations};
use crusty::sql::{get_note_by_id, add_note, list_note_titles};
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
    let test = | mock: &dyn PathOperations | {
        let note = get_note_by_id(mock, 1);
        // This tests the setup of the notes and content table from a users perspective
        // this test both create_crusty_sys_tables and populate_crusty_sys_tables
        assert!(note.title.eq("Get Started with cRusty"));
        assert!(note.body.eq("Welcome to cRusty the CLI notes app. -Ron"));
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

