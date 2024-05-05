mod setup;

use std::env;
use std::path::Path;
use crate::setup::{check_for_config, create_crusty_dir, get_home_dir, init_crusty_db};

fn main() {
    // check for a crusty home directory, if it doesn't exist show setup prompt
    let home_dir = get_home_dir();
    match check_for_config(&home_dir) {
        None => {
            println!("This is the create config path.");
            create_crusty_dir();
            init_crusty_db();
        }
        Some(conf_path) => {
            println!("This is the update config path: {:?}", conf_path)
        }
    };
}
