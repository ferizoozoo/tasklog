use std::process::exit;
use crate::{
    helper::get_home_directory,
    models::{TaskStatus, LSArgs, Task,format_string_with_color, Color,DoneArgs},
    repository,
    helper
};
pub fn handle_ls(args: &LSArgs) -> Result<(), String> {
    match helper::validate_args(args) {
        Ok(_) => {
            match repository::get_tasks(args) {
                Ok(t)  => {
                    helper::print_tasks_table(&t);
                    Ok(())
                },
                Err(e) => Err(format_string_with_color(e.as_str(), Color::Red)),
            }
        }
        Err(e) => Err(format!("Error: {}", e)),
    }
}

pub fn handel_add_task(task: Task) -> Result<(), String> {
    match helper::validate_args(&task) {
        Ok(_) => {
            let mut task = task;
            repository::save_task(&mut task).map_err(|e| format_string_with_color(format!("Error: {}",e).as_str(), Color::Red))?;

            helper::print_tasks_table(&vec![task]);
            Ok(())
        }
        Err(e) => Err(format!("Error: {}", e)),
    }
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

pub fn handle_done(done_args: DoneArgs) -> Result<(), String> {
    let task = repository::get_task_by_id(done_args.id)
            .map_err(|e|format!("Error: {}",e))?;

    if task.status == TaskStatus::Done {
        return Err("Task is already done".to_string())
    };

    repository::done_task(done_args.id)
        .map_err(|e| format!("Error: {}",e))?;

    println!("{}\n---------\n",format_string_with_color("marked task as done", Color::Green));
    helper::print_tasks_table(&vec![task]);

    Ok(())
}

