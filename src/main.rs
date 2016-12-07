// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate rustbox;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate slog;
#[macro_use] extern crate slog_scope;
extern crate slog_term;
extern crate time;
extern crate slog_stream;
extern crate slog_json;

use std::error::{Error as StdError};
use std::fs::File;
use std::io::prelude::*;
use std::env;

use slog::DrainExt;

mod errors {
    error_chain! { }
}

use errors::*;

mod viewer;

fn main() {
    // Start logger
    let log_file = File::create("ente.log").expect("Couldn't open log file");
    let file_drain = slog_stream::stream(log_file, slog_json::default());
    let logger = slog::Logger::root(file_drain.fuse(), o!());
    slog_scope::set_global_logger(logger);
    info!("Application started";
          "started_at" => format!("{}", time::now().rfc3339()));

    // Run catching errors
    if let Err(ref e) = run() {
        println!("error: {}", e);
            for e in e.iter().skip(1) {
                println!("caused by: {}", e);
            }

        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}",
                     backtrace);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Check command arguments
    let filepath = match args.len() {
        1 => bail!("You need to specify a file to open"),
        2 => &args[1],
        _ => bail!("You can only open one file"),
    };

    // Open the file
    let mut file = File::open(filepath)
          .chain_err(|| "Couldn't open file")?;
    info!("Opening file: {}", filepath);

    // Read the file and show the beginning
    let mut text = String::new();
    match file.read_to_string(&mut text) {
        Ok(_) => {},
        Err(why) => bail!("couldn't read {}: ", why.description()),
    }
    let line_count = text.lines().count();

    let filename = match filepath.to_string().split('/').last() {
        Some(name) => name.to_string(),
        None => "unknown".to_string(),
    };

    let mut viewer = viewer::Viewer::new(filename);
    viewer.init(&text, line_count);

    // Wait for keyboard events
    loop {
        match viewer.poll_event() {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    rustbox::Key::Char('q') => {
                        info!("Quitting application");
                        break;
                    }
                    rustbox::Key::Down |
                    rustbox::Key::Up |
                    rustbox::Key::PageDown |
                    rustbox::Key::PageUp => {
                        viewer.scroll(&text, line_count, key);
                    }
                    _ => { }
                }
            },
            Err(why) => bail!("Rustbox.poll_event Error {}", why.description()),
            _ => { }
        }
    }

    Ok(())
}
