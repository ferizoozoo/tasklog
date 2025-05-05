use std::env;

mod command;
mod models;

use command::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = Command::parse(args).unwrap(); // It must fail here

    if command.ls {
        println!("Listing tasks");
    }
}
