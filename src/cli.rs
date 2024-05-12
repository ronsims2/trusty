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
    pub quick: Option<String>
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