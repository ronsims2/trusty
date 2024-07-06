use std::{env, fs};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::string::ToString;
use std::time::SystemTime;
use rusqlite::Connection;
use uuid::Uuid;
use crate::errors::Errors;
use crate::render::{Printer, CrustyPrinter};

#[cfg(test)]
use mockall::*;
#[cfg(test)]
use mockall::predicate::*;

fn get_win_home_drive() -> String {
    let win_home_drive = match env::var("HOMEDRIVE") {
        Ok(val) => {
            val.to_string()
        }
        Err(_) => {
            CrustyPrinter{}.print_error(format!("{}", "Could not determine Windows user drive."));
            exit(Errors::WinUserErr as i32)
        }
    };

    win_home_drive
}

pub(crate) fn get_home_dir() -> String {
    let windows_os = "windows";
    // assume we are in a *nix env but update home path for windows if detected
    let os_fam = env::consts::FAMILY;
    let home_key = if os_fam.eq(windows_os) { "HOMEPATH" } else { "HOME" };
    let user_home = match env::var(home_key) {
        Ok(val) => {
            if os_fam.eq(windows_os) {
                let whd = get_win_home_drive();
                format!("{}/{}", whd, &val).to_string()
            } else {
                val.to_string()
            }
        }
        Err(_) => {
            CrustyPrinter{}.print_error(format!("{}", "Could not determine home path during config."));
            exit(Errors::HomePathErr as i32)
        }
    };

    user_home
}

pub fn get_crusty_directory(dir_name: String) -> PathBuf {
    let user_home = get_home_dir();
    let config_loc = format!("{}/{}", &user_home, dir_name);
    Path::new(&config_loc).to_path_buf()
}

pub(crate) fn check_for_config(home_dir: &String) -> Option<PathBuf> {
    let config_path = CrustyPathOperations{}.get_crusty_dir();

    if config_path.exists() {
        return Some(config_path)
    }
    None
}



#[cfg_attr(test, automock)]
pub trait PathOperations {
    fn get_crusty_dir(&self) -> PathBuf;
    fn get_crusty_db_path(&self) -> PathBuf;
}

pub struct CrustyPathOperations {}
impl PathOperations for CrustyPathOperations {
    fn get_crusty_dir(&self) -> PathBuf {
        get_crusty_directory(".crusty".to_string())
    }
    fn get_crusty_db_path(&self) -> PathBuf {
        let config_path = self.get_crusty_dir();
        config_path.join("crusty.db")
    }
}

pub fn create_crusty_dir(cpo: &dyn PathOperations) -> bool {
    let config_path = cpo.get_crusty_dir();
    match fs::create_dir(&config_path) {
        Ok(_) => {
            CrustyPrinter{}.println(format!("Created cRusty config at: {:?}", config_path));
        }
        Err(_) => {
            CrustyPrinter{}.print_error(format!("{}", "Could not create cRusty config directory."));
            exit(Errors::ConfigDirErr as i32)
        }
    }
    return true
}

pub fn get_db_conn(db_path: &PathBuf) -> Connection {
    Connection::open(db_path.as_path()).unwrap()
}

pub fn create_crusty_sys_tables(db_path: &PathBuf) {
    let conn = get_db_conn(db_path);
    let create_content_sql = "CREATE TABLE IF NOT EXISTS \
    content (content_id NCHAR(36) PRIMARY KEY, body TEXT);";
    let create_notes_sql = "CREATE TABLE IF NOT EXISTS notes (note_id INTEGER PRIMARY KEY AUTOINCREMENT, \
    protected BOOLEAN, title TEXT, created DATETIME, updated DATETIME, content_id NCHAR(36), trashed BOOLEAN DEFAULT FALSE, \
    CONSTRAINT fk_content_id FOREIGN KEY (content_id) REFERENCES content(content_id) ON DELETE CASCADE);";
    let create_config_sql = "CREATE TABLE IF NOT EXISTS config (key VARCHAR(36) PRIMARY KEY, value VARCHAR(140));";
    let create_app_sql = "CREATE TABLE IF NOT EXISTS app (key VARCHAR(36) PRIMARY KEY, value TEXT);";

    // populate app state
    let insert_last_touched_sql = "INSERT INTO app (key, value) VALUES ('last_touched', 0);";

    conn.execute(create_content_sql, ()).unwrap();
    conn.execute(create_notes_sql, ()).unwrap();
    conn.execute(create_config_sql, ()).unwrap();
    conn.execute(create_app_sql, ()).unwrap();
    // state inserts
    conn.execute(insert_last_touched_sql, ()).unwrap();

    CrustyPrinter{}.println(format!("{}", "Initialized empty cRusty tables."));
}

#[allow(unused)]
pub(crate) fn get_unix_epoch_ts() -> u64 {
    let ts = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    ts
}

pub(crate) fn populate_crusty_sys_tables(cpo: &dyn PathOperations) {
    let content_id = Uuid::new_v4();
    let crusty_app_id = Uuid::new_v4();
    let note_insert_sql = format!("INSERT INTO notes (title, protected, created, updated, content_id) VALUES \
    ('Get Started with cRusty', 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, '{}');", content_id);
    let content_insert_sql = format!("INSERT INTO content (content_id, body) VALUES ('{}', 'Welcome to cRusty the CLI notes app. -Ron');", content_id);
    let config_insert_app_id_sql = format!("INSERT INTO config (key, value) VALUES ('crusty_app_id', '{}');", crusty_app_id);
    let config_insert_version_sql = format!("INSERT INTO config (key, value) VALUES ('crusty_version', '{}');", env!("CARGO_PKG_VERSION"));

    let db_path = cpo.get_crusty_db_path();
    let conn = get_db_conn(&db_path);
    conn.execute(&content_insert_sql, ()).unwrap();
    conn.execute(&note_insert_sql, ()).unwrap();
    conn.execute(&config_insert_app_id_sql, ()).unwrap();
    conn.execute(&config_insert_version_sql, ()).unwrap();

    CrustyPrinter{}.println(format!("{}", "Configurations added."));
}

pub fn init_crusty_db(cpo: &dyn PathOperations) -> bool {
    let db_path = cpo.get_crusty_db_path();
    let db_created = fs::File::create(db_path.as_path());
    match db_created {
        Ok(_) => {
            create_crusty_sys_tables(&db_path);
            populate_crusty_sys_tables(cpo);
        }
        Err(_) => {
            CrustyPrinter{}.println(format!("{}", "Could not create cRusty DB."));
            exit(Errors::InitDBErr as i32)
        }
    }

    return true
}

#[cfg(test)]
mod tests {
    use std::env;
    use tempfile::tempdir;
    use super::{create_crusty_dir, CrustyPathOperations, PathOperations, MockPathOperations, get_home_dir, get_win_home_drive, init_crusty_db};
    #[test]
    fn test_get_win_home_drive() {
        let home_drive_value = "FOOBAR";
        let og_home_drive_value = env::var("HOMEDRIVE").unwrap_or("".to_string());
        env::set_var("HOMEDRIVE", home_drive_value);
        let home_drive = get_win_home_drive();
        // @todo a nice before all would work better
        env::set_var("HOMEDRIVE", og_home_drive_value);
        assert_eq!(home_drive, home_drive_value)
    }

    #[test]
    fn test_get_home_dir() {
        let windows_os = "windows";
        let os_fam = env::consts::FAMILY;
        let home_dir = get_home_dir();
        println!("Home dir: {}", home_dir.to_string());
        assert!(home_dir.len() > 0)
    }

    #[test]
    fn test_get_crusty_dir() {
        let crusty_dir = CrustyPathOperations{}.get_crusty_dir();
        assert!(crusty_dir.to_str().unwrap().contains(".crusty"))
    }

    #[test]
    fn test_create_crusty_dir() {
        let mut mock = MockPathOperations::new();
        let mock_dir = tempdir().unwrap();
        // tempdir will create a folder, you need to append a directory to this path
        // to pass to create function
        let mock_crusty_dir = mock_dir.path().join(".crusty").to_path_buf();

        mock
            .expect_get_crusty_dir()
            .return_const(mock_crusty_dir);

        let result = create_crusty_dir(&mock);
        assert!(result);
    }

    #[test]
    fn test_init_crusty_db() {
        let mock_dir = tempdir().unwrap();
        let mock_crusty_dir = mock_dir.path().join(".crusty").to_path_buf();
        let mock_crusty_db_path = mock_crusty_dir.join("crusty.db").to_path_buf();
        let mut mock = MockPathOperations::new();
        mock
            .expect_get_crusty_dir()
            .return_const(mock_crusty_dir);

        mock
            .expect_get_crusty_db_path()
            .return_const(mock_crusty_db_path);

        create_crusty_dir(&mock);
        let result = init_crusty_db(&mock);
        assert!(result);
    }
}