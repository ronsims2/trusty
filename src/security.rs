use std::process::exit;
use magic_crypt::{MagicCryptTrait, new_magic_crypt};
use regex::Regex;
use uuid::Uuid;
use crate::errors::Errors;
use crate::render::{cr_print_error, cr_println};
use crate::setup::CrustyPathOperations;
use crate::sql::{add_key_value, get_note_by_id, get_value_from_attr_table, NoteView, SimpleNoteView, update_key_value, update_note_by_note_id, update_protected_flag, update_title_by_content_id};

/**
* @compare_password - will compare what the user typed against the password saved in the database
* @confirm_password - will ask for the password 2x to make sure you typed the same one
* @fun - is passed the plain-text password as a parameter and the password should NEVER be used/seen outside the closure!!!!
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
    let saved_encrypted_password = get_value_from_attr_table(&CrustyPathOperations{}, "app", "password");
    let encrypted_password = encrypt_text(password, password);
    encrypted_password.eq(&saved_encrypted_password.value)
}

pub(crate) fn recovery_reset_password(recovery_code: &str) {
    let saved_code = get_value_from_attr_table(&CrustyPathOperations{}, "app", "recovery_code");
    let encrypted_code = encrypt_text(recovery_code, recovery_code);
    let rec_code = Some(recovery_code.to_string());
    if saved_code.value.eq(&encrypted_code) {
         set_password(true, rec_code)
    } else {
        cr_println("Invalid recovery key provided.".to_string());
    }
}


pub(crate) fn unprotect_note(note_id: usize) {
    let note = get_note_by_id(&CrustyPathOperations{}, note_id);

    if note.protected {
        update_title_by_content_id(&CrustyPathOperations{}, &note.content_id, &note.title);
        update_note_by_note_id(&CrustyPathOperations{}, note_id, &note.body);
        update_protected_flag(&CrustyPathOperations{}, note_id, false);
    } else {
        cr_println(format!("Note: {} is not encrypted.", note_id));
        exit(0);
    }
}

pub(crate) fn get_boss_key(password: &str) -> String {
    let boss_key = get_value_from_attr_table(&CrustyPathOperations{}, "app", "boss_key");
    let decrypted_boss_key = decrypt_text(password, &boss_key.value);

    decrypted_boss_key.to_string()
}

pub(crate) fn protect_note(note_id: usize) {
    let note = get_note_by_id(&CrustyPathOperations{}, note_id);

    if note.protected {
        cr_println(format!("Note: {} is already encrypted", note_id))
    } else {
        let encrypted_note = encrypt_note(&note.title, &note.body);
        update_title_by_content_id(&CrustyPathOperations{}, &note.content_id, &encrypted_note.title);
        update_note_by_note_id(&CrustyPathOperations{}, note_id, &encrypted_note.body);
        update_protected_flag(&CrustyPathOperations{}, note_id, true);

        cr_println(format!("Note: {} is now encrypted.", note_id));
    }
}

pub(crate) fn decrypt_dump(notes: &Vec<NoteView>) -> Vec<NoteView> {
    let mut decrypted_notes: Vec<NoteView> = vec![];
    let handle_decrypt = |password: &str| -> bool {
        let boss_key = get_boss_key(password);
        for note in notes {
            let decrypted_note = NoteView{
                title: decrypt_text(&boss_key, &note.title),
                body: decrypt_text(&boss_key, &note.body),
                note_id: note.note_id,
                content_id: note.content_id.to_string(),
                updated: note.updated.to_string(),
                created: note.created.to_string(),
            };

            decrypted_notes.push(decrypted_note)
        }

        return true
    };

    prompt_for_password(handle_decrypt, true, false);

    return decrypted_notes
}

pub(crate) fn set_password(update: bool, raw_recovery_code: Option<String>) {
    if update {
        cr_println("Change your password".to_string());
        let rrc = &raw_recovery_code.unwrap().to_string();
        let update_password = |pw: &str| -> bool {
            let encrypted_password = encrypt_text(pw, pw);
            let recovery_code = Uuid::new_v4().to_string();
            let encrypted_recovery_code = encrypt_text(&recovery_code, &recovery_code);
            let old_encrypted_boss_key = get_value_from_attr_table(&CrustyPathOperations{}, "app", "recovery_boss_key");
            let old_recovery_key = rrc;
            let old_decrypted_boss_key = decrypt_text(&old_recovery_key, &old_encrypted_boss_key.value);
            let new_boss_key = encrypt_text(pw, &old_decrypted_boss_key);
            let new_recovery_boss_key = encrypt_text(&recovery_code, &old_decrypted_boss_key);

            if update_key_value(&CrustyPathOperations{}, "app", "password", &encrypted_password) &&
                update_key_value(&CrustyPathOperations{}, "app", "recovery_code", &encrypted_recovery_code) &&
                update_key_value(&CrustyPathOperations{}, "app", "boss_key", &new_boss_key) &&
                update_key_value(&CrustyPathOperations{}, "app", "recovery_boss_key", &new_recovery_boss_key) {
                cr_println("Password set".to_string());
                cr_println(format!("🛟 Recovery code generated: {}", recovery_code));
                cr_println("Save your recovery code and use it to change your password if you forget it...again.".to_string());

                return true
            } else {
                cr_print_error(format!("{}", "Could not set password."));
                exit(Errors::SetPasswordErr as i32)
            }
        };

        if prompt_for_password(update_password, false, true) {
            return
        } else {
            cr_print_error(format!("{}", "Invalid password."));
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

            if add_key_value(&CrustyPathOperations{}, "app", "password", &encrypted_password) &&
                add_key_value(&CrustyPathOperations{}, "app", "recovery_code", &encrypted_recovery_code) &&
                add_key_value(&CrustyPathOperations{}, "app", "boss_key", &boss_key) &&
                add_key_value(&CrustyPathOperations{}, "app", "recovery_boss_key", &recovery_boss_key) {
                cr_println("Password set".to_string());
                cr_println(format!("🛟 Recovery code generated: {}", recovery_code));
                cr_println("Save your recovery code and use it to change your password if you forget it.".to_string());

                return true
            } else {
                cr_print_error(format!("{}", "Could not set password."));
                exit(Errors::SetPasswordErr as i32)
            }
        };

        cr_println("Set up an alpha-numeric password so that you can encrypt things 🤐".to_string());
        if prompt_for_password(insert_password, false, true) {
            return
        } else {
            cr_print_error(format!("{}", "Invalid password."));
            exit(Errors::CreatePasswordErr as i32)
        }
    }
}
