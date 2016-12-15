// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate rustbox;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate time;
extern crate slog_stream;

use std::collections::HashMap;
use std::error::Error as StdError;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::env;

use rustbox::Key;

use slog::DrainExt;

mod errors {
    error_chain!{}
}

use errors::*;

mod viewer;

const FILE_POS_LOG: usize = 65;

struct LogFormat;

impl slog_stream::Format for LogFormat {
    fn format(&self,
              io: &mut io::Write,
              rinfo: &slog::Record,
              _logger_values: &slog::OwnedKeyValueList)
              -> io::Result<()> {
        let mut msg = format!("{}: {}", rinfo.level(), rinfo.msg());
        for _ in 0..(FILE_POS_LOG - msg.len()) {
            msg.push(' ');
        }
        msg += format!("[{} : {}]\n", rinfo.file(), rinfo.line()).as_ref();
        let _ = try!(io.write_all(msg.as_bytes()));
        Ok(())
    }
}

fn main() {
    // Start logger
    let log_file = File::create("ente.log").expect("Couldn't open log file");
    let file_drain = slog_stream::stream(log_file, LogFormat);
    let logger = slog::Logger::root(file_drain.fuse(), o!());
    slog_scope::set_global_logger(logger);

    let now = time::now();
    let time_str = match now.strftime("%T %d %b %Y") {
        Ok(time) => time,
        Err(_) => now.rfc3339(),
    };
    info!("Application started (at {})", time_str);

    // Run catching errors
    if let Err(ref e) = run() {
        println!("error: {}", e);
        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}

fn fill_key_map(filepath: &str) -> HashMap<Key, viewer::Action> {
    // Defaults
    let mut actions = HashMap::new();
    actions.insert(Key::Right, viewer::Action::MoveRight);
    actions.insert(Key::Left, viewer::Action::MoveLeft);
    actions.insert(Key::Down, viewer::Action::MoveDown);
    actions.insert(Key::Up, viewer::Action::MoveUp);
    actions.insert(Key::PageUp, viewer::Action::MovePageUp);
    actions.insert(Key::PageDown, viewer::Action::MovePageDown);
    actions.insert(Key::Home, viewer::Action::MoveStartLine);
    actions.insert(Key::End, viewer::Action::MoveEndLine);
    actions.insert(Key::Char('g'), viewer::Action::GoToLine);
    actions.insert(Key::Enter, viewer::Action::Go);
    actions.insert(Key::Char('q'), viewer::Action::Quit);

    // Load config file key settings
    let mut config_file = match File::open(filepath) {
        Ok(file) => file,
        Err(_) => {
            info!("Config file {} doesn't exist", filepath);
            return actions;
        }
    };
    info!("Opening config file: {}", filepath);

    let mut text = String::new();
    match config_file.read_to_string(&mut text) {
        Ok(_) => {}
        Err(e) => {
            info!("Error reading config file: {}", e.description());
            return actions;
        }
    }

    for ln in text.lines() {
        if ln.len() == 0 || &ln[0..1] == "#" {
            continue;
        }
        let mut split = ln.split(':');
        let key = split.next().unwrap_or("NoKey");
        let k = match key.trim() {
            "G" => Key::Char('g'),
            "H" => Key::Char('h'),
            "J" => Key::Char('j'),
            "K" => Key::Char('k'),
            "L" => Key::Char('l'),
            "Q" => Key::Char('q'),
            ";" => Key::Char(';'),
            "Esc" => Key::Esc,
            _ => {
                continue;
            }
        };

        let act = split.next().unwrap_or("NoAct");
        let a = match act.trim() {
            "MoveLeft" => viewer::Action::MoveLeft,
            "MoveRight" => viewer::Action::MoveRight,
            "MoveUp" => viewer::Action::MoveUp,
            "MoveDown" => viewer::Action::MoveDown,
            "GoToLine" => viewer::Action::GoToLine,
            "Quit" => viewer::Action::Quit,
            "None" => viewer::Action::None,
            _ => {
                continue;
            }
        };

        actions.insert(k, a);
    }

    actions
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
    let mut file = File::open(filepath).chain_err(|| "Couldn't open file")?;
    info!("Opening file: {}", filepath);

    // Read the file and start a Viewer with it
    let mut text = String::new();
    match file.read_to_string(&mut text) {
        Ok(_) => {}
        Err(why) => bail!("couldn't read {}: ", why.description()),
    }
    let filename = match filepath.to_string().split('/').last() {
        Some(name) => name.to_string(),
        None => "unknown".to_string(),
    };

    let config_file = "keys.conf";
    let actions = fill_key_map(config_file);
    let mut viewer = viewer::Viewer::new(&text, filename, actions);

    // Wait for keyboard events
    match viewer.poll_event() {
        Ok(_) => Ok(()),
        Err(error) => bail!(error),
    }
}
