// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate rustbox;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate slog;
extern crate slog_term;
extern crate time;
extern crate slog_stream;
extern crate slog_json;

use std::error::{Error as StdError};
use std::default::Default;
use std::fs::File;
use std::io::prelude::*;
use std::env;

use rustbox::{Color, RustBox, OutputMode};
use rustbox::Key;

use slog::DrainExt;

mod errors {
    error_chain! { }
}

use errors::*;

fn main() {
    // Start logger
    let log_file = File::create("ente.log").expect("Couldn't open log file");
    let file_drain = slog_stream::stream(log_file, slog_json::default());
    let logger = slog::Logger::root(file_drain.fuse(), o!());
    info!(logger, "Application started";
          "started_at" => format!("{}", time::now().rfc3339()));

    // Run catching errors
    if let Err(ref e) = run(logger) {
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

fn run(logger: slog::Logger) -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut rustbox = match RustBox::init(Default::default()) {
        Ok(rustbox) => rustbox,
        Err(e) => bail!("{}", e),
    };

    // Check command arguments
    let filepath = match args.len() {
        1 => bail!("You need to specify a file to open"),
        2 => &args[1],
        _ => bail!("You can only open one file"),
    };

    // Set terminal window
    rustbox.set_output_mode(OutputMode::EightBit);
    let height = rustbox.height();
    info!(logger, "Terminal window {} lines tall", height);

    // Open the file
    let mut file = File::open(filepath)
          .chain_err(|| "Couldn't open file")?;
    info!(logger, "Opening file: {}", filepath);

    // Read the file and show as much of the beginning as possible
    let mut text = String::new();
    match file.read_to_string(&mut text) {
        Ok(_) => {},
        Err(why) => bail!("couldn't read {}: ", why.description()),
    }
    let mut lines = text.lines();
    for ln in 0 .. (height - 1) {
        if let Some(line) = lines.next() {
            rustbox.print(1, ln, rustbox::RB_BOLD, Color::White, Color::Black,
                          line);
        } else {
            break;
        }
    }

    // Add an informational status line
    rustbox.print(1, height - 1, rustbox::RB_NORMAL, Color::Black,
                  Color::Byte(0x04), "Press 'q' to quit.");

    // Display content in terminal window and wait for keyboard events
    rustbox.present();
    loop {
        match rustbox.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Key::Char('q') => {
                        info!(logger, "Quitting application");
                        break;
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
