mod cli;
mod commands;
mod logger;
mod recipe;
mod repository;
mod secret;
mod settings;
mod storage;
mod template;
mod utils;

use std::process;

fn main() {
    if let Err(err) = commands::execute_command_line(std::env::args().collect()) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}
