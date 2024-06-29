use std::io;
use std::io::{BufRead, Read};
use std::process::exit;
use clap::Parser;
use crate::errors::Errors;
use crate::render::cr_println;
use crate::security::{decrypt_note, encrypt_note};
use crate::sql::{get_last_touched_note, get_note_by_id, add_note, set_note_trash, update_note_by_content_id, update_note_by_note_id, update_title_by_content_id};
use crate::utils::{make_text_single_line, slice_text};


#[derive(Debug, Parser)]
#[command(author, version, about = "cRusty: a command Line notes app  🦀")]
pub(crate) struct Cli {
    #[arg(short, long, help = "Use this flag to specify note text (requires a title).")]
    pub note: Option<String>,
    #[arg(short, long, help = "Use this flag to a title for a note.")]
    pub title: Option<String>,
    #[arg(short, long, help = "Use this flag to specify a quick note without a title.")]
    pub quick: Option<String>,
    // must use number of args with default missing value to create flags
    //#[arg(short, long, default_missing_value = "true", num_args = 0)]
    #[arg(short, long, default_missing_value = "true", num_args = 0, help = "When specified this will read text from the standard input.  Use this to pipe in a note.")]
    pub(crate) input: Option<bool>,
    #[arg(short, long, help = "Use this flag to specify an ID to print a saved note.")]
    pub find: Option<usize>,
    #[arg(short, long, default_missing_value = "true", num_args = 0, help = "Prints a summary list of all note (default behavior if no flags are specified.")]
    pub list: Option<bool>,
    #[arg(short = 'g', long, default_missing_value = "true", num_args = 0, help = "Use this flag to find a note by piping in a menu row. Think -g like grep.")]
    pub find_from: Option<bool>,
    #[arg(short, long, default_missing_value = "true", num_args = 0, help = "Use this flag to edit the last touched note.")]
    pub edit: Option<bool>,
    #[arg(short, long, default_missing_value = "0", num_args(0..=1), help = "Use this flag to open a note by its ID.")]
    pub open: Option<usize>,
    #[arg(short = 'D', long, help = "Use this flag to delete an unprotected note by its ID.")]
    pub delete: Option<usize>,
    #[arg(short = 'F', long, help = "DANGER: This is a will indiscriminately delete a note. Use this flag to force delete a note by its ID.")]
    pub force_delete: Option<usize>,
    #[arg(short, long, default_missing_value = "true", num_args = 0, help = "Permanently delete all notes that have place din the trash.")]
    pub clean: Option<bool>,
    #[arg(long, help = "Use this flag to soft delete a note by its ID.")]
    pub trash: Option<usize>,
    #[arg(long, help = "Use this flag to restore a soft deleted a note by its ID.")]
    pub restore: Option<usize>,
    #[arg(short = 'A', long, default_missing_value = "true", num_args = 0, help = "When editing, this modifier will allow you to edit a title.")]
    pub all: Option<bool>,
    #[arg(short, long, default_missing_value = "true", num_args = 0, help = "Print all notes, very good for using grep to search or less to review.")]
    pub dump: Option<bool>,
    #[arg(long, default_missing_value = "true", num_args = 0, help = "Print a summary of statistics about your notes.")]
    pub summary: Option<bool>,
    #[arg(short = 'E', long, default_missing_value = "true", num_args = 0, help = "Specify this flag to encrypt a note.")]
    pub encrypt: Option<bool>,
    #[arg(long, help = "Use this flag to reset your password with a recovery code.")]
    pub recover: Option<String>,
    #[arg(short, long, help = "Decrypt a note and save it as plain text.")]
    pub unprotect: Option<usize>,
    #[arg(short, long, help = "Encrypt and save an existing note.")]
    pub protect: Option<usize>
}

pub(crate) fn read_from_std_in() -> Option<String> {
    let mut buffer = String::new();
    // io::stdin().read_line(&mut buffer).ok()?;
    // if !buffer.trim().is_empty() {
    //     Some(buffer.to_string())
    // } else {
    //     None
    // }

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer).unwrap();

    Some(buffer.to_string())
}

pub(crate) fn insert_note_from_std_in(title: &str, protected: bool) -> bool {
    let result = match read_from_std_in() {
        None => {
            false
        }
        Some(piped_input) => {
            if !piped_input.trim().is_empty() {
                add_note(title, &piped_input, protected);
                true
            } else {
                cr_println(format!("{}", "Input was either empty or flag was not specified, please fix your command."));
                exit(Errors::InputFlagErr as i32);
            }
        }
    };

    result
}

pub(crate) fn edit_note() {
    let note = get_last_touched_note();
    let body = note.body.as_str();
    let edited = edit::edit(body).unwrap();

    let new_body = match note.protected {
        true => {
            let encrypted_note = encrypt_note("", &edited);
            encrypted_note.body
        }
        false => {edited}
    };
    update_note_by_content_id(&note.content_id, &new_body);
}

pub(crate) fn edit_title(note_id: Option<usize>) {
    let id = note_id.unwrap_or(0);
    let note = if id > 0  {get_note_by_id(id)} else {get_last_touched_note()};
    let title = note.title.to_string();

    let edited_title = edit::edit(title).unwrap();



    let new_title = match note.protected {
        true => {
            let encrypted_note = encrypt_note(&edited_title, "");
            encrypted_note.title
        }
        false => {edited_title}
    };

    update_title_by_content_id(&note.content_id, &new_title);
}

pub(crate) fn open_note(id: usize, protected: bool) {
    if id > 0 {
        let note = get_note_by_id(id);
        let body = note.body.as_str();
        let edited = edit::edit(body).unwrap();
        update_note_by_note_id(id, &edited);
    } else {
        let draft = edit::edit("").unwrap();
        let title = slice_text(0, 128, &draft);
        add_note(&title, &draft, protected);
    }
}

pub(crate) fn trash_note(id: usize) {
    set_note_trash(id, true);
}

pub(crate) fn restore_note(id: usize) {
    set_note_trash(id, false);
}

pub(crate) fn delete_note(note_id: usize, force: bool) {
    if crate::sql::delete_note(note_id, force) {
        cr_println(format!("Note: {} deleted.", note_id))
    } else {
        cr_println(format!("Could not delete noted: {}, it may be protected or already removed.", note_id));
    }
}