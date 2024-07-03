use unicode_segmentation::UnicodeSegmentation;
use regex::Regex;
use crate::sql::{get_value_from_attr_table, SimpleNoteView};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use crate::setup::get_db_conn;

pub struct CharCount {
    ascii: usize,
    grapheme: usize,
    other: usize
}

pub(crate) fn slice_text(start: usize, stop: usize, text: &str) -> String {
    let chars = text.graphemes(true).collect::<Vec<&str>>();
    let char_count = chars.iter().count();

    return if start < stop && stop - start < char_count {
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

pub(crate) fn truncate_rich_text(text: &str, size: usize) -> String {
    let chars = text.graphemes(true).collect::<Vec<&str>>();
    let mut filtered_chars: Vec<&str> = vec!();
    let mut count = 0;

    for ch in chars {
        if ch.is_ascii() {
            count += 1
        } else {
            count += 2
        }

        if count <= size {
            filtered_chars.push(ch);
        } else {
            break;
        }
    }

    filtered_chars.join("").to_string()
}

#[cfg(test)]
mod test {
    use crate::utils::{make_text_single_line, slice_text, truncate_rich_text};

    #[test]
    fn test_slice_text() {
        let result = slice_text(0, 3, "foobar");
        assert_eq!(result, "foo");
        let result_2 = slice_text(2, 5, "foobar");
        assert_eq!(result_2, "oba");
        let result_3 = slice_text(0, 999, "foobar");
        assert_eq!(result_3, "foobar");
        let result_4 = slice_text(999, 0, "foobar");
        assert_eq!(result_4, "foobar");

    }

    #[test]
    fn test_make_text_single_line() {
        let result = make_text_single_line("Hello my name is: \r\n🤷🏾🥷 \r\nIDK ninja!");
        assert_eq!(result, "Hello my name is: 🤷🏾🥷 IDK ninja!");
        let result_2 = make_text_single_line("\r\n");
        assert_eq!(result_2, "");
        let result_3 = make_text_single_line("Hello my name is 🤷🏾🥷");
        assert_eq!(result_3, "Hello my name is 🤷🏾🥷");
    }

    #[test]
    fn test_truncate_rich_text() {
        let result_1 = truncate_rich_text("🤷🏾🥷🏽🦀😀", 2);
        assert_eq!(result_1, "🤷🏾");
        let result_2 = truncate_rich_text("🤷🏾🥷🏽🦀😀", 4);
        assert_eq!(result_2, "🤷🏾🥷🏽");
        let result_3 = truncate_rich_text("A🥷🏽B😀", 4);
        assert_eq!(result_3, "A🥷🏽B");
        let result_4 = truncate_rich_text("A🥷🏽B😀", 3);
        assert_eq!(result_4, "A🥷🏽");
        let result_5 = truncate_rich_text("🤷🏾🥷🏽🦀😀", 5);
        assert_eq!(result_5, "🤷🏾🥷🏽");
        let result_6 = truncate_rich_text("A🤷🏾BCD🥷🏽🦀E😀FGHI JKLMN", 999);
        assert_eq!(result_6, "A🤷🏾BCD🥷🏽🦀E😀FGHI JKLMN");
        let result_7 = truncate_rich_text("A🤷🏾BCD🥷🏽🦀E😀FGHI JKLMN", 0);
        assert_eq!(result_7, "");
    }

}