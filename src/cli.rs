use std::io;
use std::process::exit;
use clap::Parser;
use crate::render::cr_println;
use crate::sql::insert_note;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub(crate) struct Cli {
    #[arg(short, long)]
    pub note: Option<String>,
    #[arg(short, long)]
    pub title: Option<String>,
    #[arg(short, long)]
    pub quick: Option<String>,
    // must use number of args with default missing value to create flags
    //#[arg(short, long, default_missing_value = "true", num_args = 0)]
    #[arg(short, long)]
    pub(crate) input: Option<bool>,
    #[arg(short, long)]
    pub read: Option<usize>,
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