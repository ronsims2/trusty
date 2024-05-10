mod setup;
mod cli;
mod sql;

use std::env;
use std::fmt::Arguments;
use std::path::Path;
use clap::Parser;
use crate::cli::Cli;
use crate::setup::{check_for_config, create_crusty_dir, get_crusty_db_path, get_home_dir, init_crusty_db};
use crate::sql::insert_note;

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


    let mut new_note = Vec::new();

    if let Some(title) = args.title.as_deref() {
        println!("Title: {}", title);
        new_note.push(title);
    }

    if let Some(note) = args.note.as_deref() {
        println!("Note: {}", note);
        new_note.push(note);
    }

    if new_note.iter().count() == 2 {
        println!("2 args found");
        println!("{}::{}", new_note.get(0).unwrap(), new_note.get(1).unwrap());
    }
}
