use crate::{
    helper::get_home_directory,
    models::{CommandArgs, LSArgs, Task,format_string_with_color, Color},
    repository,
};
use crate::repository::save_task;

pub fn handle_ls(args: &LSArgs) -> Result<(), String> {
    match validate_args(args) {
        Ok(_) => {
            match repository::get_tasks(args) {
                Ok(t)  => {
                    println!("{:?}", t);
                    Ok(())
                },
                Err(e) => Err(format_string_with_color(e.as_str(), Color::Red)),
            }
        }
        Err(e) => Err(format!("Error: {}", e)),
    }
}

pub fn handel_add_task(task: Task) -> Result<(), String> {
    match validate_args(&task) {
        Ok(_) => {
            let mut task = task;
            save_task(&mut task).map_err(|e| format_string_with_color(e.as_str(), Color::Red))?;
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

    match repository::init_db(home_dir) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}
