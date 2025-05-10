use crate::{
    helper::{self, get_home_directory},
    models::{format_string_with_color, Color, DoneArgs, LSArgs, PomoTask, Task, TaskStatus},
    repository,
};
use crate::models::{AnalyzeArgs, CommandArgs};

pub fn handle_ls(args: &LSArgs) -> Result<(), String> {
    args.validate().map_err(|e| format!("Err: {}", e))?;

    let t =  repository::get_tasks(args)
        .map_err(|e| format_string_with_color(e.as_str(), Color::Red))?;

    helper::print_tasks_table(&t)
}

pub fn handel_add_task(task: Task) -> Result<(), String> {
    task.validate().map_err(|e| format!("Err: {}", e))?;

    let mut task = task;

    repository::save_task(&mut task).map_err(|e| {
        format_string_with_color(format!("Error: {}", e).as_str(), Color::Red)
    })?;

    helper::print_tasks_table(&vec![task])
}

pub fn handle_init_db() -> Result<(), String> {
    let home_dir = get_home_directory()?;

    repository::init_db(home_dir)
}

pub fn handle_analyze(analyze_args: AnalyzeArgs) -> Result<(), String>{
    Err("Not implemented yet".to_string())
}

pub fn handle_done(done_args: DoneArgs) -> Result<(), String> {
    let task = repository::get_task_by_id(done_args.id)?;

    if task.status == TaskStatus::Done {
        return Err("Task is already done".to_string());
    };

    repository::done_task(done_args.id).map_err(|e| format!("Error: {}", e))?;

    println!("{}\n\n",format_string_with_color("marked task as done", Color::Green));

    helper::print_tasks_table(&vec![task])
}

pub fn handle_pomodoro(pomo_task: PomoTask) -> Result<(), String> {
    pomo_task.validate().map_err(|e| format!("Error: {}", e))?;

    let mut pomo_value = pomo_task;
    // save to repo
    repository::add_pomodoro(&mut pomo_value)
        .map_err(|e| format!("Error: {}", e))?;

    // clear screen
    helper::clear_terminal_screen()
        .map_err(|e| format!("Error: {}", e))?;
    // display the count-down

    helper::print_count_down_screen(&pomo_value)
}