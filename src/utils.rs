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