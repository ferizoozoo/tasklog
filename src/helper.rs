use std::env;
use crate::models::{format_string_with_color, Color, Task, AppState, PomodoroEvent, PomoTask};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue, style::Print,
    terminal::{self, ClearType, Clear},
};
use std::{
    io::{stdout, Stdout, Write},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};
use crate::handlers;

const BOX_WIDTH:u16=40;
const BOX_HEIGHT:u16 = 4;

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


pub fn print_tasks_table(tasks: &Vec<Task>) -> Result<(),String> {
    if tasks.is_empty() {
        return Err(format_string_with_color("NOT_FOUND", Color::Red));
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

    Ok(())
}


fn format_time(seconds: u64) -> String {
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

fn center_text(text: &str, width: usize) -> String {
    let padding = if text.len() < width {
        (width - text.len()) / 2
    } else {
        0
    };

    format!("{}{}", " ".repeat(padding), text)
}



pub fn clear_terminal_screen() -> Result<(), String> {
    let mut stout = std::io::stdout();
    let res = execute!(
        stout,
        Clear(ClearType::All),
        cursor::MoveTo(0,0),
    );

    match res {
        Ok(_) => Ok(()),
        Err(e) =>  Err(e.to_string()),
    }
}


pub fn draw_ui(stdout: &mut Stdout, state: &AppState) -> Result<(), String> {
    queue!(stdout, terminal::Clear(ClearType::All)).map_err(|e| e.to_string())?;

    let box_start_col = if state.term_width >= BOX_WIDTH {
        (state.term_width - BOX_WIDTH) / 2
    } else {
        0
    };

    let box_start_row = if state.term_height >= BOX_HEIGHT {
        (state.term_height - BOX_HEIGHT) / 2
    } else {
        0
    };

    let content_inner_width = (BOX_WIDTH.saturating_sub(2)) as usize;

    let total_seconds = state.current_time.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    let time_str = format!("{:02}:{:02}", minutes, seconds);

    let border_line = "-".repeat(BOX_WIDTH as usize);

    let title_to_display = if state.title.chars().count() > content_inner_width {
        state.title.chars().take(content_inner_width).collect::<String>()
    } else {
        state.title.to_string()
    };

    let mut title_padded = format!("{:^width$}", title_to_display, width = content_inner_width);
    title_padded = format_string_with_color(title_padded.as_str(), Color::Cyan);
    let title_line_content = format!("|{}|", title_padded);

    let mut time_padded = format!("{:^width$}", time_str, width = content_inner_width);
    time_padded = format_string_with_color(time_padded.as_str(), Color::Yellow);
    
    let time_line_content = format!("|{}|", time_padded);


    let res = queue!(
        stdout,
        cursor::MoveTo(box_start_col, box_start_row),
        Print(&border_line),
        cursor::MoveTo(box_start_col, box_start_row + 1),
        Print(&title_line_content),
        cursor::MoveTo(box_start_col, box_start_row + 2),
        Print(&time_line_content),
        cursor::MoveTo(box_start_col, box_start_row + 3),
        Print(&border_line)
    );

    match res {
        Ok(_) => stdout.flush().map_err(|e| e.to_string()),
        Err(e) =>  Err(e.to_string()),
    }
}

pub fn run_timer_thread(
    initial_duration: Duration,
    time_update_tx: Sender<Duration>,
    quit_rx: Receiver<()>,
) {
    let mut current_duration = initial_duration;
    loop {
        // Check for quit signal non-blockingly.
        if quit_rx.try_recv().is_ok() {
            break; // Exit if quit signal received.
        }

        // Send the current time to the main thread.
        if time_update_tx.send(current_duration).is_err() {
            break; // Main thread likely terminated.
        }

        // Stop if countdown reaches zero.
        if current_duration == Duration::ZERO {
            break;
        }

        // Wait for one second.
        // To make the quit signal check more responsive, sleep in smaller intervals.
        for _ in 0..10 { // Sleep for 10 * 100ms = 1 second
            if quit_rx.try_recv().is_ok() {
                return; // Exit immediately if quit signal received during sleep.
            }
            thread::sleep(Duration::from_millis(100));
        }

        current_duration = current_duration.saturating_sub(Duration::from_secs(1));
    }
}

pub fn run_event_thread(event_tx: Sender<PomodoroEvent>, quit_rx: Receiver<()>) {
    loop {
        // Check for quit signal non-blockingly.
        if quit_rx.try_recv().is_ok() {
            break; // Exit if quit signal received.
        }

        // Poll for terminal events with a timeout.
        if event::poll(Duration::from_millis(200)).unwrap_or(false) {
            match event::read() {
                // For killing the app, use the 'q' key or 'ctrl+c'
                Ok(Event::Key(KeyEvent {
                                  code: KeyCode::Char('q'), ..
                              }))
                | Ok(Event::Key(KeyEvent {
                                    code: KeyCode::Char('c'),
                                    modifiers: KeyModifiers::CONTROL, ..
                                })) => {
                    if event_tx.send(PomodoroEvent::Quit).is_err() {
                        break; // Main thread likely terminated.
                    }
                    // Once quit is sent, this thread can exit.
                    break;
                }

                // For resizing the terminal
                Ok(Event::Resize(width, height)) => {
                    if event_tx.send(PomodoroEvent::Resize(width, height)).is_err() {
                        break; // Main thread likely terminated.
                    }
                }

                // any error should kill the event thread
                Err(_) => {
                    // Error reading event, could signal this or just break.
                    let _ = event_tx.send(PomodoroEvent::Quit); // Signal main to quit on error
                    break;
                }
                _ => {} // Ignore other events.
            }
        }
        // No explicit sleep here as event::poll has a timeout.
    }
}


