use std::io;
use clap::Parser;

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
    if (!buffer.is_empty()) {
        println!("Standard in: {}", buffer.as_str());
        Some(buffer.to_string())
    } else {
        None
    }
}