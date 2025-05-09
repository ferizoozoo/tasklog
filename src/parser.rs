use crate::repository::get_tasks;

use super::handlers::handle_ls;
use super::models::{format_string_with_color, Cli, Color, CommandArgs, Commands};
use super::{handlers, models};
use clap::Parser;

pub fn execute() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => match handlers::handle_init_db() {
            Ok(_) => println!("Database initialized successfully."),
            Err(err) => {
                eprintln!("{}", format_string_with_color(err.as_str(), Color::Red));
            }
        },
        Commands::LS(args) => match handle_ls(&args) {
            Ok(_) => {
                let tasks = get_tasks(args);
                println!("{:?}", tasks);
                ()
            }
            Err(err) => {
                eprintln!("{}", format_string_with_color(err.as_str(), Color::Red));
            }
        },
        Commands::Add(args) => match args.validate() {
            Ok(_) => {
                println!("{:?}", args);
            }
            Err(err) => {
                eprintln!("{}", format_string_with_color(err.as_str(), Color::Red));
            }
        },
        Commands::Analyze(args) => {
            println!("Analyzing tasks");
            println!("Since: {}", args.days);
        }
        Commands::Done(args) => {
            println!("Marking task {:?} as done", args);
        }
        Commands::Pomo(args) => {
            println!("Starting pomo");
            println!("{:?}", args);
        }
    }
}
