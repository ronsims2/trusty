use std::io;
use clap::Parser;
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
    #[arg(short, long, default_missing_value = "true", num_args = 0)]
    pub(crate) input: Option<bool>
}

pub(crate) fn read_from_std_in() -> Option<String> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).ok()?;
    if !buffer.is_empty() {
        println!("Standard in: {}", buffer.as_str());
        Some(buffer.to_string())
    } else {
        None
    }
}

pub(crate) fn insert_note_from_std_in(flag_set: bool, title: &str) -> bool {
    let result = match read_from_std_in() {
        None => {
            println!("No stdin found");
            false
        }
        Some(piped_input) => {
            if !piped_input.trim().is_empty() && flag_set {
                println!("The piped input: {}", piped_input);
                insert_note(title, &piped_input, false);
                true
            } else {
                println!("Input was either empty or flag was not specified, please fix your command.");
                false
            }
        }
    };

    result
}