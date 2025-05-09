use crate::{
    helper::get_home_directory,
    models::{CommandArgs, LSArgs},
    repository::init_db,
};
use std::env;

pub fn handle_ls(args: &LSArgs) -> Result<(), String> {
    match validate_args(args) {
        Ok(_) => {
            println!("inside handler {:?}", args);
            Ok(())
        }
        Err(e) => Err(format!("Error: {}", e)),
    }
}

fn validate_args<T: CommandArgs>(args: &T) -> Result<(), String> {
    args.validate()
}

pub fn handle_init_db() -> Result<(), String> {
    let home_dir = match get_home_directory() {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    match init_db(home_dir) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}
