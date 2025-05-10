use std::process::exit;
use crate::repository::get_tasks;

use super::models::{format_string_with_color, Cli, Color, CommandArgs, Commands};
use super::{handlers};
use clap::Parser;

pub fn execute() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => match handlers::handle_init_db() {
            Ok(_) => println!("Database initialized successfully."),
            Err(err) =>{
                eprintln!("{}", format_string_with_color(err.as_str(), Color::Red));
                exit(1)
            } 
        },
        Commands::LS(args) => match handlers::handle_ls(&args) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", format_string_with_color(err.as_str(), Color::Red));
                exit(1)
            }
        },
        Commands::Add(task) => match  handlers::handel_add_task(task) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", format_string_with_color(err.as_str(), Color::Red));
                exit(1)
            },
        },
        Commands::Analyze(args) => match handlers::handle_analyze(args){
            Ok(_) => {},
            Err(err) => {
                eprintln!("{}", format_string_with_color(err.as_str(), Color::Red));
                exit(1)
            }
        }
        Commands::Done(args) => match handlers::handle_done(args){
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", format_string_with_color(err.as_str(), Color::Red));
                exit(1)
            }
        }
        Commands::Pomo(args) => match handlers::handle_pomodoro(args){
            Ok(_) => {},
            Err(err) => {
                eprintln!("{}", format_string_with_color(err.as_str(), Color::Red));
                exit(1)
            }
        }
    }
}
