use std::process::exit;
use magic_crypt::{MagicCryptTrait, new_magic_crypt};
use regex::Regex;
use crate::render::cr_println;
use crate::setup::set_password;
use crate::sql::{get_note_by_id, get_value_from_attr_table, SimpleNoteView, update_note_by_note_id, update_protected_flag, update_title_by_content_id};

/**
* @compare_password - will compare what the user typed against the password saved in the database
* @confirm_password - will ask for the password 2x to make sure you typed the same one
* @fun - is passed the password and should NEVER be used outside the closure!!!!
*/
pub(crate) fn prompt_for_password<F>(mut fun: F, compare_password_to_db: bool, confirm_password: bool) -> bool where F: FnMut(&str) -> bool {
    let mut attempts = 0;
    while attempts < 2  {
        let password = rpassword::prompt_password("Enter password: ").unwrap();
        let password2 = if confirm_password {
            rpassword::prompt_password("Enter your password again: ").unwrap()
        } else {
            password.clone()
        };

        if password.eq(&password2) && validate_password(&password) {
            if compare_password_to_db  {
                if check_password(&password) {
                    if fun(&password) {
                        return true
                    }
                }
            } else {
                if fun(&password) {
                    return true
                }
            }
        }

        cr_println("Password incorrect, try again.".to_string());
        attempts += 1;
    }

    cr_println("Password incorrect.".to_string());
    false
}

pub(crate) fn decrypt_note(title: &str, note: &str) -> SimpleNoteView {
    let mut unencrypted_title = "".to_string();
    let mut unencrypted_note = "".to_string();

    let handle_decrypt = | password: &str| -> bool {
        let decrypted_boss_key = get_boss_key(password);
        unencrypted_title = decrypt_text(&decrypted_boss_key, title);
        unencrypted_note = decrypt_text(&decrypted_boss_key, note);

        return true
    };

    prompt_for_password(handle_decrypt, true, false);

    return SimpleNoteView {
        title: unencrypted_title,
        body: unencrypted_note,
        content_id: "0".to_string(),
        protected: true
    }
}

pub(crate) fn encrypt_note(title: &str, note: &str) -> SimpleNoteView {
    let mut encrypted_title = "".to_string();
    let mut encrypted_body = "".to_string();

    let handle_encrypt = |password: &str| -> bool {
        let decrypted_boss_key = get_boss_key(password);
        encrypted_title = encrypt_text(&decrypted_boss_key, title);
        encrypted_body = encrypt_text(&decrypted_boss_key, note);

        return true
    };

    prompt_for_password(handle_encrypt, true, false);

    return SimpleNoteView {
        title: encrypted_title,
        body: encrypted_body,
        content_id: "0".to_string(),
        protected: true
    }
}

pub(crate) fn validate_password(password: &str) -> bool {
    // password may not be blank, may be alphanumeric and at least 4 characters
    if password.trim().is_empty() {
        return false
    }

    let re = Regex::new(r"^[A-Za-z0-9]+$").unwrap();

    if !re.is_match(password) {
        return false
    }

    if password.len() < 3 {
        return false
    }

    true
}

pub(crate) fn encrypt_text(key: &str, text: &str) -> String {
    let mc = new_magic_crypt!(key, 256);
    mc.encrypt_str_to_base64(text)
}

pub(crate) fn decrypt_text(key: &str, text: &str) -> String {
    let mc = new_magic_crypt!(key, 256);
    mc.decrypt_base64_to_string(text).unwrap()
}

pub(crate) fn check_password(password: &str) -> bool {
    let saved_encrypted_password = get_value_from_attr_table("app", "password");
    let encrypted_password = encrypt_text(password, password);
    encrypted_password.eq(&saved_encrypted_password.value)
}

pub(crate) fn recovery_reset_password(recovery_code: &str) {
    let saved_code = get_value_from_attr_table("app", "recovery_code");
    let encrypted_code = encrypt_text(recovery_code, recovery_code);
    let rec_code = Some(recovery_code.to_string());
    if saved_code.value.eq(&encrypted_code) {
         set_password(true, rec_code)
    } else {
        cr_println("Invalid recovery key provided.".to_string());
    }
}


pub(crate) fn unprotect_note(note_id: usize) {
    let note = get_note_by_id(note_id);

    if note.protected {
        update_title_by_content_id(&note.content_id, &note.title);
        update_note_by_note_id(note_id, &note.body);
        update_protected_flag(note_id, false);
    } else {
        cr_println(format!("Note: {} is not encrypted.", note_id));
        exit(0);
    }
}

pub(crate) fn get_boss_key(password: &str) -> String {
    let boss_key = get_value_from_attr_table("app", "boss_key");
    let decrypted_boss_key = decrypt_text(password, &boss_key.value);

    decrypted_boss_key.to_string()
}