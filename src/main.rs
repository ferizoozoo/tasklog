mod command;
mod models;

use clap::Parser;
use command::Cli;
use command::Commands;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::LS(args)  => {
            println!("Listing up to tasks");
            println!("Limit: {}", args.limit);
            println!("Days: {}", args.days);
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
        },
        Commands::Analyze(args) => {
            println!("Analyzing tasks");
            println!("Since: {}", args.days);

        },
        Commands::Done { id } => {
            println!("Marking task {} as done", id);
        },
        Commands::Pomo(args) => {
            match args.command {
                Some(command::PomoCommands::Logs(log_args)) => {
                    println!("Viewing pomodoro logs");
                    if let Some(since) = log_args.since {
                        println!("Since: {}", since);
                    }
                },
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
            }
        },
    }
}
