use std::env;
use std::env::temp_dir;
use std::path::PathBuf;
use mockall::*;
use mockall::predicate::*;
use tempfile::tempdir;
use crusty::*;
use crusty::setup::{create_crusty_dir, get_crusty_directory, init_crusty_db, PathOperations};
use crusty::sql::get_note_by_id;

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
fn test_populate_crusty_sys_tables() {
    let test = | mock: &dyn PathOperations | {
        let note = get_note_by_id(mock, 1);
        println!("NOTE: TITLE: {}", note.title);
        // This tests the setup of the notes and content table from a users perspective
        // this test both create_crusty_sys_tables and populate_crusty_sys_tables
        assert!(note.title.eq("Get Started with cRusty"));
        assert!(note.body.eq("Welcome to cRusty the CLI notes app. -Ron"));
    };

    create_test_db(test);
}