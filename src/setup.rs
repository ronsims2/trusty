use std::{env, fs};
use std::env::VarError;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::string::ToString;
use clap::builder::Str;
use clap::Parser;
use sqlite::Connection;


fn get_win_home_drive() -> String {
    let win_home_drive = match env::var("HOMEDRIVE") {
        Ok(val) => {
            val.to_string()
        }
        Err(_) => {
            println!("Could not determine Windows user drive.");
            exit(501)
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
            println!("Could not determine home path during config.");
            exit(502)
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
            println!("Created cRusty config at: {:?}", config_path);
        }
        Err(_) => {
            println!("Could not create cRusty config directory.");
            exit(503)
        }
    }
}

pub(crate) fn get_crusty_db_path() -> PathBuf {
    let config_path = get_crusty_dir();
    config_path.join("crusty.db")
}

pub(crate) fn get_crusty_db_conn() -> Connection {
    let db_path = get_crusty_db_path();
    sqlite::open(db_path.as_path()).unwrap()
}

pub(crate) fn create_main_crusty_table() {
    let conn = get_crusty_db_conn();
    let sql = "CREATE TABLE IF NOT EXISTS \
    content (content_id NCHAR(36) PRIMARY KEY, body TEXT); \
    CREATE TABLE IF NOT EXISTS notes (note_id INTEGER PRIMARY KEY AUTOINCREMENT, \
    title VARCHAR(64), created DATETIME, updated DATETIME, content_id NCHAR(36), \
    CONSTRAINT fk_content_id FOREIGN KEY (content_id) REFERENCES content(content_id));";

    conn.execute(sql).unwrap();
}

pub(crate) fn init_crusty_db() {
    let db_path = get_crusty_db_path();
    let db_created = fs::File::create(db_path.as_path());
    match db_created {
        Ok(_) => {
            create_main_crusty_table();
            println!("cRusty DB created.");
        }
        Err(_) => {
            println!("Could not create cRusty DB.");
        }
    }
}