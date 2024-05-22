use std::io;
use std::process::exit;
use clap::Parser;
use crate::render::cr_println;
use crate::sql::{get_last_touched_note, insert_note, update_note};



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
    #[arg(short, long, help = "Use this flag to open a note by its ID.")]
    pub open: Option<usize>
}

pub(crate) fn read_from_std_in() -> Option<String> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).ok()?;
    if !buffer.trim().is_empty() {
        Some(buffer.to_string())
    } else {
        None
    }
}

pub(crate) fn insert_note_from_std_in(title: &str) -> bool {
    let result = match read_from_std_in() {
        None => {
            false
        }
        Some(piped_input) => {
            if !piped_input.trim().is_empty() {
                insert_note(title, &piped_input, false);
                true
            } else {
                cr_println(format!("{}", "Input was either empty or flag was not specified, please fix your command."));
                exit(505);
            }
        }
    };

    result
}

pub(crate) fn edit_note() {
    let note = get_last_touched_note();
    let body = note.body.as_str();
    let edited = edit::edit(body).unwrap();
    update_note(&note.id, &edited);
}