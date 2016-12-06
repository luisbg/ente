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
use std::default::Default;
use std::fs::File;
use std::io::prelude::*;
use std::env;

use rustbox::{Color, RustBox, OutputMode, EventResult};
use rustbox::Key;

use slog::DrainExt;

mod errors {
    error_chain! { }
}

use errors::*;

struct Viewer {
    rustbox: RustBox,
    height: usize
}

impl Viewer {
    fn new() -> Viewer {
        let mut rustbox = RustBox::init(Default::default()).unwrap();
        let height = rustbox.height();
        rustbox.set_output_mode(OutputMode::EightBit);
        info!("Terminal window height: {}", height);

        Viewer {
            rustbox: rustbox,
            height: height,
        }
    }

    fn display_chunk(&mut self, text: &String, start: usize) -> Result<()> {
        self.rustbox.clear();

        if start > text.lines().count() {
            warn!("Line {} past EOF", start);
            return Err("End of file".into());
        }

        let mut lines = text.lines().skip(start - 1);
        for ln in 0 .. (self.height - 1) {
            if let Some(line) = lines.next() {
                self.rustbox.print(1, ln, rustbox::RB_BOLD, Color::White,
                                   Color::Black, line);
            } else {
                info!("Displayed range {} : {} lines", start,
                   start + ln - 1);
                return Ok(());
            }
        }

        info!("Displayed range {} : {} lines", start, start + self.height - 2);
        Ok(())
    }

    fn update(&mut self) {
        // Add an informational status line
        self.rustbox.print(1, self.height - 1, rustbox::RB_NORMAL, Color::Black,
                      Color::Byte(0x04), "Press 'q' to quit.");

        self.rustbox.present();
    }

    fn poll_event(&mut self) -> EventResult {
        self.rustbox.poll_event(false)
    }
}

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
    let mut cur = 1;

    let mut viewer = Viewer::new();

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
    match viewer.display_chunk(&text, cur) {
        Ok(_) => viewer.update(),
        Err(_) => {}
    }

    // Wait for keyboard events
    loop {
        match viewer.poll_event() {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Key::Char('q') => {
                        info!("Quitting application");
                        break;
                    }
                    Key::Down => {
                        cur += 1;
                        match viewer.display_chunk(&text, cur) {
                            Ok(_) => viewer.update(),
                            Err(_) => { cur -= 1}
                        }
                    }
                    Key::Up => {
                        if cur > 1 {
                            cur -= 1;
                        }
                        match viewer.display_chunk(&text, cur) {
                            Ok(_) => viewer.update(),
                            Err(_) => {}
                        }
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
