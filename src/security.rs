use magic_crypt::{MagicCryptTrait, new_magic_crypt};
use regex::Regex;
use crate::render::cr_println;
use crate::sql::{get_value_from_attr_table, SimpleNoteView};

// compare_password will check against the db
pub(crate) fn prompt_password<F>(fun: F, compare_password: bool) -> bool where F: FnOnce(&str) -> bool {
    /* @todo refactor, maybe split into 2 prompt functions 1 should be for creation,
         the other should be for decryption */
    let mut attempts = 0;
    while attempts < 2  {
        let password = rpassword::prompt_password("Create your password: ").unwrap();
        let password2 = rpassword::prompt_password("Enter your password again: ").unwrap();

        if password.eq(&password2) && validate_password(&password) {
            return if compare_password && check_password(&password) {
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

pub(crate) fn decrypt_note(title: &str, note: &str) ->SimpleNoteView {
    let mut unencrypted_title = "".to_string();
    let mut unencrypted_note = "".to_string();
    let handle_decrypt = | password: &str| -> bool {
        unencrypted_title = decrypt_text(password, title);
        unencrypted_note = decrypt_text(password, note);
        return true;
    };

    prompt_password(handle_decrypt, true);

    return SimpleNoteView {
        title: unencrypted_title,
        body: unencrypted_note,
        content_id: "0".to_string(),
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