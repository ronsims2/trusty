use unicode_segmentation::UnicodeSegmentation;
use regex::Regex;
use crate::sql::{get_value_from_attr_table, SimpleNoteView};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use crate::setup::get_crusty_db_conn;

pub(crate) fn slice_text(start: usize, stop: usize, text: &str) -> String {
    let chars = text.graphemes(true).collect::<Vec<&str>>();
    let char_count = chars.iter().count();

    return if stop - start < char_count {
        let sample = &chars[start..stop];
        sample.join("").to_string()
    } else {
        text.to_string()
    }
}

pub(crate) fn make_text_single_line(text: &str) -> String {
    let lines = text.lines();
    let mut new_text = lines.map(|ln| {
        let content = ln.trim();
        format!("{} ", content)
    });

    new_text.collect::<String>().trim().to_string()
}

pub(crate) fn validate_password(password: &str) -> bool {
    println!("The password is: {}", password.to_string());
    // password may not be blank, may be alphanumeric and at least 4 characters
    if password.trim().is_empty() {
        println!("The password is empty");
        return false
    }

    let re = Regex::new(r"^[A-Za-z0-9]+$").unwrap();

    if !re.is_match(password) {
        println!("The password does not match the pattern");
        return false
    }

    if password.len() < 3 {
        println!("The password is too short");
        return false
    }

    true
}

pub(crate) fn encrypt_text(key: &str, text: &str) -> String {
    let mc = new_magic_crypt!(key, 256);
    mc.encrypt_str_to_base64(text)
}

pub(crate) fn check_password(password: &str) {
    let saved_encrypted_password = get_value_from_attr_table("app", "password");
    let encrypted_password = encrypt_text(password, password);

    println!("saved: {} | attempted: {}", saved_encrypted_password.value, encrypted_password)
}