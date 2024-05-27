mod setup;
mod cli;
mod sql;
mod render;
mod utils;

use std::env;
use std::fmt::Arguments;
use std::path::Path;
use clap::Parser;
use crate::cli::{Cli, edit_note, edit_title, insert_note_from_std_in, open_note, read_from_std_in, restore_note, trash_note};
use crate::render::{print_note_summary, print_simple_note};
use crate::setup::{check_for_config, create_crusty_dir, get_crusty_db_path, get_home_dir, init_crusty_db};
use crate::sql::{delete_note, empty_trash, get_note_by_id, get_note_from_menu_line, insert_note, list_note_titles};
use crate::utils::slice_text;

fn main() {
    // check for a crusty home directory, if it doesn't exist show setup prompt
    let home_dir = get_home_dir();
    let conf_loc = match check_for_config(&home_dir) {
        None => {
            create_crusty_dir();
            init_crusty_db();
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
    let hard_delete = args.hard_delete;
    let force_delete = args.force_delete;
    let clean = args.clean;
    let trash = args.trash;
    let restore = args.restore;
    let all = args.all;

    if input.is_some() {
        let title_val = title.unwrap_or("Untitled");
        let result = insert_note_from_std_in(title_val);
        if result {
            return
        }
    }

    // if there is a title and note param insert a proper note
    if title.is_some() && note.is_some() {
        insert_note(title.unwrap(), note.unwrap(), false);
        return
    }

    if find.is_some() {
        let note = get_note_by_id(find.unwrap());
        print_simple_note(note);
        return
    }

    if find_from.is_some() {
        let note = get_note_from_menu_line();
        print_simple_note(note);
        return
    }

    // add an untitled quick note, this needs to stay near the bottom
    if quick_note.is_some() && title.is_none() && note.is_none() {
        let note = quick_note.unwrap();
        let title = slice_text(0, 64, note);
        insert_note(title.as_str(), note, false);
        return
    }

    if edit.is_some() {
        if all.is_some() {
            edit_title();
        }
        edit_note();
        return
    }

    if open.is_some() {
        let note_id = open.unwrap();
        open_note(note_id);

        return;
    }

    if hard_delete.is_some() {
        let note_id = hard_delete.unwrap();
        delete_note(note_id, false);
        list_note_titles();
        return
    }

    if force_delete.is_some() {
        let note_id = force_delete.unwrap();
        delete_note(note_id, true);
        list_note_titles();
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
        list_note_titles();
        return
    }


    // if there is no input at all show the menu
    list_note_titles()
}
