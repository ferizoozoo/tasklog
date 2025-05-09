use std::env;
use crate::models::{format_string_with_color, Color, CommandArgs, Task};

pub fn get_home_directory() -> Result<String, String> {
    if let Ok(path) = env::var("HOME") {
        return Ok(path);
    }

    if let Ok(path) = env::var("USERPROFILE") {
        return Ok(path);
    }

    if let (Ok(drive), Ok(path)) = (env::var("HOMEDRIVE"), env::var("HOMEPATH")) {
        return Ok(format!("{}{}", drive, path));
    }

    Err("Could not get the home directory".to_string())
}


pub fn validate_args<T: CommandArgs>(args: &T) -> Result<(), String> {
    args.validate()
}


pub fn print_tasks_table(tasks: &Vec<Task>) {
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