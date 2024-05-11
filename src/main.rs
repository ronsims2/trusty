mod setup;
mod cli;
mod sql;

use std::env;
use std::fmt::Arguments;
use std::path::Path;
use clap::Parser;
use crate::cli::Cli;
use crate::setup::{check_for_config, create_crusty_dir, get_crusty_db_path, get_home_dir, init_crusty_db};
use crate::sql::{insert_note, list_note_titles};

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

    // if there is a title and note param insert a proper note
    if title.is_some() && note.is_some() {
        println!("2 args found");
        // println!("{}::{}", new_note.get(0).unwrap(), new_note.get(1).unwrap());
        insert_note(title.unwrap(), note.unwrap(), false)
    }

    // add an untitled quick note
    let quick_note = args.quick.as_deref();
    if (quick_note.is_some() && title.is_none() && note.is_none()) {
        insert_note("Untitled", quick_note.unwrap(), false)
    }
    // if there is no input at all show the menu
    list_note_titles()
}
