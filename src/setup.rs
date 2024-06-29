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
use crate::security::{decrypt_note, decrypt_text, prompt_for_password};
use crate::sql::{add_key_value, get_value_from_attr_table, update_key_value};
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

pub(crate) fn set_password(update: bool, raw_recovery_code: Option<String>) {
    if update {
        cr_println("Change your password".to_string());
        let rrc = &raw_recovery_code.unwrap().to_string();
        let update_password = |pw: &str| -> bool {
            let encrypted_password = encrypt_text(pw, pw);
            let recovery_code = Uuid::new_v4().to_string();
            let encrypted_recovery_code = encrypt_text(&recovery_code, &recovery_code);
            let old_encrypted_boss_key = get_value_from_attr_table("app", "recovery_boss_key");
            let old_recovery_key = rrc;
            let old_decrypted_boss_key = decrypt_text(&old_recovery_key, &old_encrypted_boss_key.value);
            let new_boss_key = encrypt_text(pw, &old_decrypted_boss_key);
            let new_recovery_boss_key = encrypt_text(&recovery_code, &old_decrypted_boss_key);

            if update_key_value("app", "password", &encrypted_password) &&
                update_key_value("app", "recovery_code", &encrypted_recovery_code) &&
                update_key_value("app", "boss_key", &new_boss_key) &&
                update_key_value("app", "recovery_boss_key", &new_recovery_boss_key) {
                cr_println("Password set".to_string());
                cr_println(format!("🛟 Recovery code generated: {}", recovery_code));
                cr_println("Save your recovery code and use it to change your password if you forget it...again.".to_string());

                return true
            } else {
                cr_println(format!("{}", "Could not set password."));
                exit(Errors::SetPasswordErr as i32)
            }
        };

        if prompt_for_password(update_password, false, true) {
            return
        } else {
            cr_println(format!("{}", "Invalid password."));
            exit(Errors::CreatePasswordErr as i32)
        }
    } else {
        let insert_password = |pw: &str| -> bool {
            let encrypted_password = encrypt_text(pw, pw);
            let recovery_code = Uuid::new_v4().to_string();
            let encrypted_recovery_code = encrypt_text(&recovery_code, &recovery_code);
            let raw_boss_key = Uuid::new_v4().to_string();
            let boss_key = encrypt_text(pw, &raw_boss_key);
            let recovery_boss_key = encrypt_text(&recovery_code, &raw_boss_key);

            if add_key_value("app", "password", &encrypted_password) &&
                add_key_value("app", "recovery_code", &encrypted_recovery_code) &&
                add_key_value("app", "boss_key", &boss_key) &&
                add_key_value("app", "recovery_boss_key", &recovery_boss_key) {
                cr_println("Password set".to_string());
                cr_println(format!("🛟 Recovery code generated: {}", recovery_code));
                cr_println("Save your recovery code and use it to change your password if you forget it.".to_string());

                return true
            } else {
                cr_println(format!("{}", "Could not set password."));
                exit(Errors::SetPasswordErr as i32)
            }
        };

        cr_println("Set up an alpha-numeric password so that you can encrypt things 🤐".to_string());
        if prompt_for_password(insert_password, false, true) {
            return
        } else {
            cr_println(format!("{}", "Invalid password."));
            exit(Errors::CreatePasswordErr as i32)
        }
    }
}