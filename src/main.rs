#[macro_use]
extern crate cursive;
#[macro_use]
extern crate serde;

use std::io::Write;
use std::{backtrace, fs::File, path::PathBuf, process::exit};

use application::{Application, setup_logging};
use config::set_configuration_base_path;
use log::error;
use respot::program_arguments;

mod application;
mod authentication;
mod command;
mod config;
mod events;
mod library;
mod lyrics;
mod model;
mod queue;
mod serialization;
mod spotify;
mod theme;
mod traits;
mod ui;
mod utils;

#[cfg(unix)]
mod ipc;

#[cfg(feature = "mpris")]
mod mpris;

#[cfg(feature = "notify")]
mod notification;

/// Register a custom panic handler to write backtraces to a file called `backtrace.log` inside the
/// user's cache directory.
fn register_backtrace_panic_handler() {
    // During most of the program, Cursive is responsible for drawing to the
    // tty. Since stdout probably doesn't work as expected during a panic, the
    // backtrace is written to a file at $USER_CACHE_DIR/respot/backtrace.log.
    std::panic::set_hook(Box::new(|panic_info| {
        // A panic hook will prevent the default panic handler from being
        // called. An unwrap in this part would cause a hard crash of respot.
        // Don't unwrap/expect/panic in here!
        if let Ok(backtrace_log) = config::try_proj_dirs() {
            let mut path = backtrace_log.cache_dir;
            path.push("backtrace.log");
            if let Ok(mut file) = File::create(path) {
                writeln!(file, "{}", backtrace::Backtrace::force_capture()).unwrap_or_default();
                writeln!(file, "{panic_info}").unwrap_or_default();
            }
        }
    }));
}

/// Print platform info like which platform directories will be used.
fn info() -> Result<(), String> {
    let user_configuration_directory = config::user_configuration_directory();
    let user_cache_directory = config::user_cache_directory();

    println!(
        "USER_CONFIGURATION_PATH {}",
        user_configuration_directory
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or("not found".into())
    );
    println!(
        "USER_CACHE_PATH {}",
        user_cache_directory
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or("not found".into())
    );

    #[cfg(unix)]
    {
        let user_runtime_directory = utils::user_runtime_directory();
        println!(
            "USER_RUNTIME_PATH {}",
            user_runtime_directory
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or("not found".into())
        );
    }

    Ok(())
}

fn main() -> Result<(), String> {
    // Set a custom backtrace hook that writes the backtrace to a file instead of stdout, since
    // stdout is most likely in use by Cursive.
    register_backtrace_panic_handler();

    // Parse the command line arguments.
    let matches = program_arguments().get_matches();

    // Enable debug logging to a file if specified on the command line.
    if let Some(filename) = matches.get_one::<PathBuf>("debug") {
        setup_logging(filename).expect("logger could not be initialized");
    }

    // Set the configuration base path. All configuration files are read/written relative to this
    // path.
    set_configuration_base_path(matches.get_one::<PathBuf>("basepath").cloned());

    match matches.subcommand() {
        Some(("info", _subcommand_matches)) => info(),
        Some((_, _)) => unreachable!(),
        None => {
            // Create the application.
            let mut application =
                match Application::new(matches.get_one::<String>("config").cloned()) {
                    Ok(application) => application,
                    Err(error) => {
                        eprintln!("{error}");
                        error!("{error}");
                        exit(-1);
                    }
                };

            // Start the application event loop.
            application.run()
        }
    }?;

    Ok(())
}
