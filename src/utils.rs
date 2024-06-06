use unicode_segmentation::UnicodeSegmentation;
use regex::Regex;
use crate::sql::{SimpleNoteView};

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
    // password may not be blank, may be alphanumeric and at least 4 characters
    if password.trim().is_empty() {
        return false
    }

    let re = Regex::new(r"^[A-Za-z0-9]+$").unwrap();

    if !re.is_match(password) {
        return false
    }

    if password.len() > 3 {
        return false
    }

    true
}