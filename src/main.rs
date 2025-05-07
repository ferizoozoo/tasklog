mod command;
mod handlers;
mod models;
mod repository;

use clap::Parser;
use command::Cli;
use command::{format_string_with_color, Color, Commands};
use handlers::handle_ls;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => match handlers::handle_init_db() {
            Ok(_) => println!("Database initialized successfully."),
            Err(err) => {
                eprintln!("{}", format_string_with_color(err.as_str(), Color::Red));
            }
        },
        Commands::LS(args) => match handle_ls(args) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", format_string_with_color(err.as_str(), Color::Red));
            }
        },
        Commands::Add(args) => {
            println!("Adding new task: {}", args.title);
            if let Some(due_date) = args.due_date {
                println!("Due date: {}", due_date);
            }
            println!("Priority: {:?}", args.priority);
            if let Some(category) = args.category {
                println!("Category: {}", category);
            }
        }
        Commands::Analyze(args) => {
            println!("Analyzing tasks");
            println!("Since: {}", args.days);
        }
        Commands::Done { id } => {
            println!("Marking task {} as done", id);
        }
        Commands::Pomo(args) => match args.command {
            Some(command::PomoCommands::Logs(log_args)) => {
                println!("Viewing pomodoro logs");
                if let Some(since) = log_args.since {
                    println!("Since: {}", since);
                }
            }
            None => {
                println!("Starting pomodoro session");
                if let Some(title) = args.title {
                    println!("Title: {}", title);
                }
                if let Some(duration) = args.duration {
                    println!("Duration: {}", duration);
                }
                if let Some(category) = args.category {
                    println!("Category: {}", category);
                }
            }
        },
    }
}
