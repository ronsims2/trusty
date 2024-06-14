use std::{env, fs};
use std::env::VarError;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::string::ToString;
use std::time::SystemTime;
use clap::builder::Str;
use clap::Parser;
use rusqlite::{Connection, params, Params};
use uuid::Uuid;
use crate::errors::Errors;
use crate::render::cr_println;
use crate::security::prompt_password;
use crate::sql::add_key_value;
use crate::security::{check_password, encrypt_text, validate_password};


fn get_win_home_drive() -> String {
    let win_home_drive = match env::var("HOMEDRIVE") {
        Ok(val) => {
            val.to_string()
        }
        Err(_) => {
            cr_println(format!("{}", "Could not determine Windows user drive."));
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
            cr_println(format!("{}", "Could not determine home path during config."));
            exit(Errors::HomePathErr as i32)
        }
    };

    user_home
}

pub(crate) fn get_crusty_dir() -> PathBuf {
    let user_home = get_home_dir();
    let config_loc = format!("{}/.crusty", &user_home);
    Path::new(&config_loc).to_path_buf()
}

pub(crate) fn check_for_config(home_dir: &String) -> Option<PathBuf> {
    let config_path = get_crusty_dir();

    if config_path.exists() {
        return Some(config_path)
    }
    None
}

pub(crate) fn create_crusty_dir() {
    let config_path = get_crusty_dir();
    match fs::create_dir(&config_path) {
        Ok(_) => {
            cr_println(format!("Created cRusty config at: {:?}", config_path));
        }
        Err(_) => {
            cr_println(format!("{}", "Could not create cRusty config directory."));
            exit(Errors::ConfigDirErr as i32)
        }
    }
}

pub(crate) fn get_crusty_db_path() -> PathBuf {
    let config_path = get_crusty_dir();
    config_path.join("crusty.db")
}

pub(crate) fn get_crusty_db_conn() -> Connection {
    let db_path = get_crusty_db_path();
    Connection::open(db_path.as_path()).unwrap()
}

pub(crate) fn create_crusty_sys_tables() {
    let conn = get_crusty_db_conn();
    let create_content_sql = "CREATE TABLE IF NOT EXISTS \
    content (content_id NCHAR(36) PRIMARY KEY, body TEXT);";
    let create_notes_sql = "CREATE TABLE IF NOT EXISTS notes (note_id INTEGER PRIMARY KEY AUTOINCREMENT, \
    protected BOOLEAN, title VARCHAR(64), created DATETIME, updated DATETIME, content_id NCHAR(36), trashed BOOLEAN DEFAULT FALSE, \
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

    cr_println(format!("{}", "Initialized empty cRusty tables."));
}

pub(crate) fn get_unix_epoch_ts() -> u64 {
    let ts = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    ts
}

pub(crate) fn populate_crusty_sys_tables() {
    let content_id = Uuid::new_v4();
    let crusty_app_id = Uuid::new_v4();
    let note_insert_sql = format!("INSERT INTO notes (title, protected, created, updated, content_id) VALUES \
    ('Get Started with cRusty', 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, '{}');", content_id);
    let content_insert_sql = format!("INSERT INTO content (content_id, body) VALUES ('{}', 'Welcome to cRusty the CLI notes app. -Ron');", content_id);
    let config_insert_app_id_sql = format!("INSERT INTO config (key, value) VALUES ('crusty_app_id', '{}');", crusty_app_id);
    let config_insert_version_sql = format!("INSERT INTO config (key, value) VALUES ('crusty_version', '{}');", env!("CARGO_PKG_VERSION"));

    let conn = get_crusty_db_conn();
    conn.execute(&content_insert_sql, ()).unwrap();
    conn.execute(&note_insert_sql, ()).unwrap();
    conn.execute(&config_insert_app_id_sql, ()).unwrap();
    conn.execute(&config_insert_version_sql, ()).unwrap();
    cr_println(format!("{}", "Configurations added."));


}

pub(crate) fn init_crusty_db() {
    let db_path = get_crusty_db_path();
    let db_created = fs::File::create(db_path.as_path());
    match db_created {
        Ok(_) => {
            create_crusty_sys_tables();
            populate_crusty_sys_tables();
        }
        Err(_) => {
            cr_println(format!("{}", "Could not create cRusty DB."));
        }
    }
}

pub(crate) fn set_password(update: bool, current_count: i32) {
    let insert_password = |pw: &str| -> bool {
        let encrypted_password = encrypt_text(pw, pw);
        if add_key_value("app", "password", &encrypted_password) {
            cr_println("Password set".to_string());
            return true
        } else {
            cr_println(format!("{}", "Could not set password."));
            exit(Errors::SetPasswordErr as i32)
        }
    };

    if prompt_password(insert_password, false) {
        return
    } else {
        cr_println(format!("{}", "Invalid password."));
        exit(Errors::CreatePasswordErr as i32)
    }
}