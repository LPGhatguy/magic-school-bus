extern crate clap;
extern crate crossterm;
extern crate open;

pub mod action;
pub mod input_state;
pub mod state;
pub mod terminal_context;
pub mod ui;
pub mod virtual_screen;
pub mod virtual_screen_buffer;

use std::{
    env,
    panic,
    process,
    path::PathBuf,
};

use clap::{App, Arg};

use crate::{
    action::Action,
    input_state::InputState,
    state::State,
    terminal_context::TerminalContext,
    virtual_screen::VirtualScreen,
};

struct AppConfig {
    print_working_directory: bool,
    start_dir: PathBuf,
}

fn start(config: &AppConfig) {
    let mut state = State::new(config.start_dir.clone());
    let mut input_state = InputState::new();
    let context = TerminalContext::init();
    let (width, height) = context.get_terminal_size();
    let mut screen = VirtualScreen::new(width, height);

    loop {
        ui::nudge_state(&mut state, &screen);
        screen.render_prepare(&context);
        ui::render(&state, &input_state, &mut screen);
        screen.commit(&context);

        if let Some(action) = input_state.process_input(&context) {
            state.process_action(action);

            if action == Action::Quit {
                break;
            }

            if action == Action::DebugDumpVisible {
                eprintln!("{}", screen.show_current_buffer());
            }
        }
    }

    drop(context);

    if config.print_working_directory {
        eprintln!("{}", state.working_directory.display());
    }
}

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

    let result = panic::catch_unwind(move || start(&config));

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