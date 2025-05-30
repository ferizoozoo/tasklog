use crate::helper::{draw_ui, run_event_thread, run_timer_thread};
use crate::models::{
    AnalyzeArgs, AppState, CommandArgs, LSType, PomoStatus, PomodoroEvent, TableRow,
};
use crate::{
    helper::{self, get_home_directory},
    models::{format_string_with_color, Color, DoneArgs, LSArgs, PomoTask, Task, TaskStatus},
    repository,
};
use chrono::Local;
use crossterm::cursor;
use crossterm::{execute, terminal};
use std::io::stdout;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// When we have to map types 10000000000000000000 times in rust the most simple tasks like passing
/// to a fucking function, why the fuck they say Rust's performance is good?????????
pub fn handle_ls(args: &LSArgs) -> Result<(), String> {
    args.validate().map_err(|e| format!("Err: {}", e))?;

    let t: Vec<Box<dyn TableRow>> = match args.ls_type {
        LSType::Task => repository::get_tasks(args)
            .map_err(|e| format_string_with_color(e.as_str(), Color::Red))
            .unwrap()
            .into_iter()
            .map(|t| Box::new(t) as Box<dyn TableRow>)
            .collect(),

        LSType::Pomo => repository::get_pomodoro(args)
            .map_err(|e| format_string_with_color(e.as_str(), Color::Red))
            .unwrap()
            .into_iter()
            .map(|t| Box::new(t) as Box<dyn TableRow>)
            .collect(),
    };

    let due_date = Local::now() + chrono::Duration::days(args.days as i64);
    let text = format_string_with_color(
        format!(
            "Results tasks with due_date as {} or before:\n",
            due_date.format("%Y-%m-%d")
        )
        .as_str(),
        Color::Green,
    );

    println!("{}", text);
    helper::print_tables(&t)
}

pub fn handel_add_task(task: Task) -> Result<(), String> {
    task.validate().map_err(|e| format!("Err: {}", e))?;

    let mut task = task;

    repository::save_task(&mut task)
        .map_err(|e| format_string_with_color(format!("Error: {}", e).as_str(), Color::Red))?;

    helper::print_tasks_table(&vec![task])
}

pub fn handle_init_db() -> Result<(), String> {
    let home_dir = get_home_directory()?;

    repository::init_db(home_dir)
}

pub fn handle_analyze(analyze_args: AnalyzeArgs) -> Result<(), String> {
    analyze_args.validate().map_err(|e| format!("Err: {}", e))?;

    let analysis_volumes = repository::get_analysis(&analyze_args)
        .map_err(|e| format!("Err: {}", e))
        .unwrap()
        .into_iter()
        .map(|t| Box::new(t) as Box<dyn TableRow>)
        .collect();

    helper::print_tables(&analysis_volumes).map_err(|e| format!("Error printing tables: {}", e))?;
    Ok(())
}

pub fn handle_done(done_args: DoneArgs) -> Result<(), String> {
    let task = repository::get_task_by_id(done_args.id)?;

    if task.status == TaskStatus::Done {
        return Err("Task is already done".to_string());
    };

    repository::done_task(done_args.id).map_err(|e| format!("Error: {}", e))?;

    println!(
        "{}\n\n",
        format_string_with_color("marked task as done", Color::Green)
    );

    helper::print_tasks_table(&vec![task])
}

pub fn handle_pomodoro(pomo_task: PomoTask) -> Result<(), String> {
    pomo_task.validate()?;

    let mut pomo_value = pomo_task;
    repository::add_pomodoro(&mut pomo_value)?;

    helper::clear_terminal_screen()?;

    control_terminal(&mut pomo_value)
}

pub fn control_terminal(pomo_task: &mut PomoTask) -> Result<(), String> {
    let mut stdout = stdout();

    // --- Setup Terminal ---
    terminal::enable_raw_mode().map_err(|e| e.to_string())?;
    // Enter alternate screen to keep main terminal clean. Hide cursor.
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).map_err(|e| e.to_string())?;

    let initial_duration = pomo_task.duration.to_time_duration();

    let (term_width, term_height) = terminal::size().map_err(|e| e.to_string())?;

    let mut app_state = AppState {
        title: pomo_task.title.clone(),
        term_width,
        term_height,
        current_time: initial_duration,
        quited: false,
    };

    let (time_update_tx, time_update_rx) = mpsc::channel::<Duration>();
    let (event_tx, event_rx) = mpsc::channel::<PomodoroEvent>();

    let (timer_quit_tx, timer_quit_rx) = mpsc::channel::<()>();
    let (event_thread_quit_tx, event_thread_quit_rx) = mpsc::channel::<()>();

    let timer_handle = {
        let time_update_tx_clone = time_update_tx.clone();
        thread::spawn(move || {
            run_timer_thread(initial_duration, time_update_tx_clone, timer_quit_rx);
        })
    };

    let event_handle = {
        let event_tx_clone = event_tx.clone();
        thread::spawn(move || {
            run_event_thread(event_tx_clone, event_thread_quit_rx);
        })
    };

    draw_ui(&mut stdout, &app_state).map_err(|e| e.to_string())?;

    loop {
        // Process incoming messages non-blockingly.
        // Order of checking: events first, then time updates.

        // Check for application events (Resize, Quit)
        match event_rx.try_recv() {
            Ok(PomodoroEvent::Quit) => {
                app_state.quited = true;
                break; // Exit main loop.
            }
            Ok(PomodoroEvent::Resize(new_width, new_height)) => {
                println!("Resizing to {}x{}", new_width, new_height);
                app_state.term_width = new_width;
                app_state.term_height = new_height;
                draw_ui(&mut stdout, &app_state)?;
            }
            Err(mpsc::TryRecvError::Empty) => {
                // No event, continue.
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // Event thread terminated, should also quit.
                app_state.quited = true;
                break;
            }
        }

        // Check for time updates from the timer thread.
        match time_update_rx.try_recv() {
            Ok(new_time) => {
                app_state.current_time = new_time;
                draw_ui(&mut stdout, &app_state)?;
                if app_state.current_time == Duration::ZERO {
                    // Optional: Display "Time's up!" message here before breaking.
                    // For now, just break the loop.
                    break;
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                // No time update, continue.
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // Timer thread terminated. If not due to time up, this might be an issue.
                // If time is already zero, this is expected.
                if app_state.current_time != Duration::ZERO {
                    // Timer thread died unexpectedly.
                }
                break;
            }
        }

        // Small sleep to prevent main loop from busy-waiting excessively.
        // This also determines the responsiveness to channel messages.
        thread::sleep(Duration::from_millis(50));
    }

    let _ = timer_quit_tx.send(());
    let _ = event_thread_quit_tx.send(());

    // Wait for threads to finish (optional but good practice).
    let _ = timer_handle.join();
    let _ = event_handle.join();

    // Restore terminal: Leave alternate screen, show cursor.
    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show).map_err(|e| e.to_string())?;

    terminal::disable_raw_mode().map_err(|e| e.to_string())?;

    pomo_task.end_time = Local::now();

    if app_state.quited {
        pomo_task.status = PomoStatus::Paused;
    } else {
        pomo_task.status = PomoStatus::Finished
    }

    repository::update_pomodoro(pomo_task)
}
