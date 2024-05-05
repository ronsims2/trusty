mod setup;

use std::env;
use crate::setup::create_config;

fn main() {
    // check for a crusty home directory, if it doesn't exist show setup prompt
    create_config()
}
