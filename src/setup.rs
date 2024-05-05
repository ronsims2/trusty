use std::env;
use std::env::VarError;
use std::path::Path;
use clap::Parser;


pub(crate) fn create_config() {
    // assume we are in a *nix env but update home path for windows if detected
    let os_fam = env::consts::FAMILY;
     let home_key = if &os_fam.eq("windows") {"%HOMEPATH%"} else {"HOME"};
     let mut user_home = match env::var(home_key) {
         Ok(val) => {
             println!("The users is on a: {} system, and their home dir is at: {}", os_fam, val);
             val.to_string()
         },
         Err(err) => {
             println!("Error: {}", err);
             "".to_string()
         }
     };


    println!("The home directory is at: {}", user_home)
    // look for config first in home dir
    // let home_loc = homedir::get_home();
    // let conf_loc = Path::new().is_file();
}