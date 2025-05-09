use std::process::exit;
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
                    print_tasks_table(&t);
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
            println!("Adding task: {:?}", task);
            save_task(&mut task).map_err(|e| format_string_with_color(format!("Error: {}",e).as_str(), Color::Red))?;

            println!("Task added successfully");
            print_tasks_table(&vec![task]);
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



fn print_tasks_table(tasks: &Vec<Task>) {
    if tasks.is_empty() {
        eprintln!("{}",format_string_with_color("NOT_FOUND", Color::Red));
       return;
    }

    // Define the column headers
    let headers = ["id", "title", "status", "due_date", "priority", "category"];

    // Calculate the width of each column based on content
    let mut col_widths = vec![
        headers[0].len(),  // id
        headers[1].len(),  // title
        headers[2].len(),  // status
        headers[3].len(),  // due_date
        headers[4].len(),  // priority
        headers[5].len(),  // category
    ];

    // Update the column widths based on the task data
    for task in tasks {
        // ID column width
        col_widths[0] = col_widths[0].max(task.id.to_string().len());

        // Title column width
        col_widths[1] = col_widths[1].max(task.title.len());

        // Status column width
        col_widths[2] = col_widths[2].max(format!("{:?}", task.status).len());

        // Due date column width
        col_widths[3] = col_widths[3].max(task.due_date.format("%Y-%m-%d").to_string().len());

        // Priority column width
        col_widths[4] = col_widths[4].max(format!("{:?}", task.priority).len());

        // Category column width
        let category_str = task.category.as_ref().map_or("", |s| s.as_str());
        col_widths[5] = col_widths[5].max(category_str.len());
    }

    // Print header row with proper padding
    print!("| ");
    for (i, header) in headers.iter().enumerate() {
        print!("{:<width$} | ", header, width = col_widths[i]);
    }
    println!();

    // Print separator row
    print!("|");
    for width in &col_widths {
        print!("-{}-|", "-".repeat(*width));
    }
    println!();

    // Print each task row
    for task in tasks {
        print!("| ");
        // ID column
        print!("{:<width$} | ", task.id, width = col_widths[0]);

        // Title column
        print!("{:<width$} | ", task.title, width = col_widths[1]);

        // Status column
        print!("{:<width$} | ", format!("{:?}", task.status), width = col_widths[2]);

        // Due date column
        print!("{:<width$} | ", task.due_date.format("%Y-%m-%d"), width = col_widths[3]);

        // Priority column
        print!("{:<width$} | ", format!("{:?}", task.priority), width = col_widths[4]);

        // Category column
        let category_str = task.category.as_ref().map_or("", |s| s.as_str());
        print!("{:<width$} | ", category_str, width = col_widths[5]);

        println!();
    }
}
