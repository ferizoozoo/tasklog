use crate::command::{CommandArgs, LSArgs};

pub fn handle_ls(args: LSArgs) -> Result<(), String> {
    match validate_args(&args) {
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
    // Initialize the database
    // This is a placeholder for the actual database initialization logic

    // Run the Scehma models

    Ok(())
}
