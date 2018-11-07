extern crate clap;
extern crate crossterm;
extern crate open;

pub mod app;
pub mod state;
pub mod terminal_context;
pub mod virtual_screen;

use std::{
    env,
    panic,
    process,
};

use crate::app::AppConfig;

fn main() {
    let mut config = AppConfig {
        print_working_directory: false,
        start_dir: env::current_dir().unwrap(),
    };

    match env::args().nth(1) {
        Some(flag) => {
            if flag == "--pwd" {
                config.print_working_directory = true;
            } else {
                eprintln!("Unknown argument {}", flag);
                eprintln!("Valid arguments are:");
                eprintln!("    --pwd: Print the current directory to stderr at close.");
                process::exit(1);
            }
        },
        _ => {},
    }

    let result = panic::catch_unwind(move || app::start(config));

    if let Err(error) = result {
        let message = match error.downcast_ref::<&str>() {
            Some(message) => message.to_string(),
            None => match error.downcast_ref::<String>() {
                Some(message) => message.clone(),
                None => "<no message>".to_string(),
            },
        };

        eprintln!("The Magic School Bus crashed!\n{}", message);

        process::exit(1);
    }
}