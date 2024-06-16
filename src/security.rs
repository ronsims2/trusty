use magic_crypt::{MagicCryptTrait, new_magic_crypt};
use regex::Regex;
use crate::render::cr_println;
use crate::sql::{get_value_from_attr_table, SimpleNoteView};

// compare_password will check against the db
pub(crate) fn prompt_for_password<F>(fun: F, compare_password_to_db: bool, confirm_password: bool) -> bool where F: FnOnce(&str) -> bool {
    let mut attempts = 0;
    while attempts < 2  {
        let password = rpassword::prompt_password("Enter password: ").unwrap();
        let password2 = if confirm_password {
            rpassword::prompt_password("Enter your password again: ").unwrap()
        } else {
            password.clone()
        };

        if password.eq(&password2) && validate_password(&password) {
            return if compare_password_to_db && check_password(&password) {
                fun(&password)
            } else {
                fun(&password)
            }
        }
        cr_println("Password incorrect, try again.".to_string());
        attempts += 1;
    }
    false
}

pub(crate) fn decrypt_note(title: &str, note: &str) -> SimpleNoteView {
    let mut unencrypted_title = "".to_string();
    let mut unencrypted_note = "".to_string();
    let handle_decrypt = | password: &str| -> bool {
        unencrypted_title = decrypt_text(password, title);
        unencrypted_note = decrypt_text(password, note);
        return true;
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
        encrypted_title = encrypt_text(password, title);
        encrypted_body = encrypt_text(password, note);
        return true;
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