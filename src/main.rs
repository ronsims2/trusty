mod setup;
mod cli;
mod sql;
mod render;
mod utils;
mod errors;
mod security;

use clap::Parser;
use security::set_password;
use crate::cli::{Cli, delete_note, edit_note, edit_title, insert_note_from_std_in, open_note, restore_note, trash_note};
use crate::render::{cr_println, print_app_summary, print_dump, print_simple_note};
use crate::setup::{check_for_config, create_crusty_dir, get_crusty_db_path, get_home_dir, init_crusty_db};
use crate::sql::{add_note, dump_notes, empty_trash, get_note_by_id, get_note_from_menu_line, get_summary, list_note_titles};
use crate::utils::slice_text;
use crate::security::{protect_note, recovery_reset_password, unprotect_note};

fn main() {
    // check for a crusty home directory, if it doesn't exist show setup prompt
    let home_dir = get_home_dir();
    let conf_loc = match check_for_config(&home_dir) {
        None => {
            create_crusty_dir();
            init_crusty_db();
            set_password(false, None);
            get_crusty_db_path()
        }
        Some(conf_path) => {
            conf_path
        }
    };

    // read the args
    let args = Cli::parse();

    let title = args.title.as_deref();
    let note = args.note.as_deref();
    let quick_note = args.quick.as_deref();
    let input = args.input;
    let find = args.find;
    let find_from = args.find_from;
    let edit = args.edit;
    let open = args.open;
    let delete = args.delete;
    let force_delete = args.force_delete;
    let clean = args.clean;
    let trash = args.trash;
    let restore = args.restore;
    let all = args.all;
    let dump = args.dump;
    let summary = args.summary;
    let encrypted = args.encrypt;
    let recover = args.recover;
    let unprotect = args.unprotect;
    let protect = args.protect;
    let dump_protected = args.dump_protected;

    let should_encrypt_note = encrypted.unwrap_or(false);

    if find_from.is_some() {
        let note = get_note_from_menu_line();
        print_simple_note(note);
        return
    }

    // reset password flow
    if recover.is_some() {
        let recovery_code = recover.unwrap();
        recovery_reset_password(&recovery_code);
        return
    }

    if protect.is_some() {
        let note_id = protect.unwrap_or(0);
        protect_note(note_id);
        return
    }

    if unprotect.is_some() {
        let note_id = unprotect.unwrap_or(0);
        unprotect_note(note_id);
        return
    }

    if summary.is_some() {
        let summary = get_summary();
        print_app_summary(summary);
        return
    }

    if input.is_some() {
        let title_val = title.unwrap_or("Untitled");
        insert_note_from_std_in(title_val, should_encrypt_note);
        return
    }

    // if there is a title and note param insert a proper note
    // @todo this could replace the quick note command if we unwrap+or for the title
    if title.is_some() && note.is_some() {
        add_note(title.unwrap(), note.unwrap(), should_encrypt_note);
        return
    }

    if find.is_some() {
        let note = get_note_by_id(find.unwrap());
        print_simple_note(note);
        return
    }

    // add an untitled quick note, this needs to stay near the bottom
    if quick_note.is_some() && title.is_none() && note.is_none() {
        let note = quick_note.unwrap();
        let title = slice_text(0, 128, note);
        add_note(title.as_str(), note, should_encrypt_note);
        return
    }

    if edit.is_some() {
        if all.is_some() {
            edit_title(None);
        }
        edit_note();
        return
    }

    if open.is_some() {
        let note_id = open.unwrap();
        if all.is_some() {
            edit_title(Some(note_id));
        }
        open_note(note_id, should_encrypt_note);
        return
    }

    if delete.is_some() {
        let note_id = delete.unwrap();
        delete_note(note_id, false);
        return
    }

    if force_delete.is_some() {
        let note_id = force_delete.unwrap();
        delete_note(note_id, true);
        return
    }

    if clean.is_some() {
        empty_trash();
        list_note_titles();
        return
    }

    if trash.is_some() {
        let note_id = trash.unwrap();
        trash_note(note_id);
        list_note_titles();
        return
    }

    if restore.is_some() {
        let note_id = restore.unwrap();
        restore_note(note_id);
        cr_println(format!("Note: {} restored", note_id));
        return
    }

    if dump.is_some() {
        let notes = dump_notes(false);
        print_dump(notes);
        return
    }

    if dump_protected.is_some() {
        let notes = dump_notes(true);
        print_dump(notes);
        return
    }


    // if there is no input at all show the menu
    // @todo pass flag encrypt message here
    list_note_titles()
}
