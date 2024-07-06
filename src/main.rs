mod setup;
mod cli;
mod sql;
mod render;
mod utils;
mod errors;
mod security;

use clap::Parser;
use security::set_password;
use crate::cli::{Cli, edit_note, edit_title, insert_note_from_std_in, open_note};
use crate::render::{print_app_summary, print_dump, print_simple_note, TrustyPrinter, Printer};
use crate::setup::{check_for_config, create_trusty_dir, TrustyPathOperations, PathOperations, get_home_dir, init_trusty_db};
use crate::sql::{add_note, delete_note, dump_notes, empty_trash, get_note_by_id, get_note_from_menu_line, get_summary, list_note_titles, restore_note, trash_note};
use crate::utils::slice_text;
use crate::security::{protect_note, recovery_reset_password, unprotect_note};

fn main() {
    // check for a trusty home directory, if it doesn't exist show setup prompt
    let cpo = TrustyPathOperations {};
    let cr_print = TrustyPrinter {};
    let home_dir = get_home_dir();
    let conf_loc = match check_for_config(&home_dir) {
        None => {
            create_trusty_dir(&cpo);
            init_trusty_db(&cpo);
            set_password(false, None);
            cpo.get_trusty_db_path()
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
        let note = get_note_from_menu_line(&cpo);
        print_simple_note(&cr_print, note);
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
        let summary = get_summary(&cpo);
        print_app_summary(&cr_print, summary);
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
        add_note(&cpo, title.unwrap(), note.unwrap(), should_encrypt_note);
        return
    }

    if find.is_some() {
        let note = get_note_by_id(&cpo, find.unwrap());
        print_simple_note(&cr_print, note);
        return
    }

    // add an untitled quick note, this needs to stay near the bottom
    if quick_note.is_some() && title.is_none() && note.is_none() {
        let note = quick_note.unwrap();
        let title = slice_text(0, 128, note);
        add_note(&cpo, title.as_str(), note, should_encrypt_note);
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
        open_note(&cpo, note_id, should_encrypt_note);
        return
    }

    if delete.is_some() {
        let note_id = delete.unwrap();
        delete_note(&cpo, note_id, false);
        return
    }

    if force_delete.is_some() {
        let note_id = force_delete.unwrap();
        delete_note(&cpo, note_id, true);
        return
    }

    if clean.is_some() {
        empty_trash(&TrustyPathOperations {});
        list_note_titles(&cpo, &cr_print);
        return
    }

    if trash.is_some() {
        let note_id = trash.unwrap();
        trash_note(&cpo, note_id);
        list_note_titles(&cpo, &cr_print);
        return
    }

    if restore.is_some() {
        let note_id = restore.unwrap();
        restore_note(&cpo, note_id);
        cr_print.println(format!("Note: {} restored", note_id));
        return
    }

    if dump.is_some() {
        let notes = dump_notes(&cpo,false);
        print_dump(&cr_print, notes);
        return
    }

    if dump_protected.is_some() {
        let notes = dump_notes(&cpo,true);
        print_dump(&cr_print, notes);
        return
    }


    // if there is no input at all show the menu
    // @todo pass flag encrypt message here
    list_note_titles(&cpo, &cr_print)
}
