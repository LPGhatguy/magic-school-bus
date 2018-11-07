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
    path::PathBuf,
};

use clap::{App, Arg};

use crate::app::AppConfig;

fn main() {
    let matches = App::new("Magic School Bus")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))

        .arg(Arg::with_name("START_DIR")
            .help("The directory to start in, defaulting to the current working directory.")
            .index(1))

        .arg(Arg::with_name("pwd")
            .long("pwd")
            .help("Prints the current directory to stderr when closing."))

        .get_matches();

    let start_dir = match matches.value_of("START_DIR") {
        Some(start_dir) => PathBuf::from(start_dir),
        None => env::current_dir().unwrap(),
    };

    let print_working_directory = matches.is_present("pwd");

    let config = AppConfig {
        print_working_directory,
        start_dir,
    };

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