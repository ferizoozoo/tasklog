mod command;
mod handlers;
mod models;
mod repository;
mod parser;

use crate::parser::execute;

fn main() {
   execute()
}
