// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate rustbox;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate time;
extern crate slog_stream;
extern crate rustc_serialize;
extern crate docopt;

use std::error::Error as StdError;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;

use std::env;
use slog::DrainExt;
use std::collections::HashMap;
use docopt::Docopt;

mod errors {
    error_chain!{}
}

use errors::*;

mod viewer;
mod model;
mod keyconfig;

#[cfg(test)]
mod modeltest;

const FILE_POS_LOG: usize = 65;

struct LogFormat;

const USAGE: &'static str = "
Ente text editor.

Usage:
  ente FILE [--keyconfig=<kc>] [--hide-line-num]
  ente (-h | --help)
  ente --version

  Options:
    -h --help          Show this screen.
    --version          Show version.
    --keyconfig=<kc>   Key configurationg file.
    --hide-line-num    Hide line numbers.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_file: String,
    flag_keyconfig: String,
    flag_hidelinenum: bool,
}

impl slog_stream::Format for LogFormat {
    fn format(&self,
              io: &mut io::Write,
              rinfo: &slog::Record,
              _logger_values: &slog::OwnedKeyValueList)
              -> io::Result<()> {
        let mut msg = format!("{}: {}", rinfo.level(), rinfo.msg());
        if msg.len() < FILE_POS_LOG {
            for _ in 0..(FILE_POS_LOG - msg.len()) {
                msg.push(' ');
            }

            msg += format!("[{} : {}]\n", rinfo.file(), rinfo.line()).as_ref();
        } else {
            msg += "\n";
        }
        try!(io.write_all(msg.as_bytes()));
        Ok(())
    }
}

fn main() {
    let args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.parse())
        .unwrap_or_else(|e| e.exit());

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

    let mut key_config_file = args.get_str("--keyconfig").to_string();
    if key_config_file.is_empty() {
        // No config file set
        key_config_file = match env::home_dir() {
            Some(mut path) => {
                path.push(".config/ente/keys.conf");
                path.to_str().unwrap().to_string()
            }
            None => String::from("keys.conf"),
        }
    }
    let actions = keyconfig::fill_key_map(key_config_file.as_ref());

    let hide_line_num = args.get_bool("--hide-line-num");

    // Run catching errors
    if let Err(ref e) = run(args.get_str("FILE"), actions, !hide_line_num) {
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

fn open_file(filepath: &str) -> String {
    // Open the file
    let mut text = String::new();

    let path = Path::new(filepath);
    if path.is_dir() {
        panic!("Can't open a folder. {}", filepath);
    }

    if path.is_file() {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => panic!("File {} does not exist. Creating it.", filepath),
        };

        info!("Opening file: {}", filepath);

        // Read the file into a String
        match file.read_to_string(&mut text) {
            Ok(_) => {}
            Err(error) => panic!("couldn't read {}: ", error.description()),
        }
    } else {
        info!("Creating new file: {}", filepath);
        File::create(path).expect("Couldn't create file");
    }

    if text.lines().count() == 0 {
        text.push('\n');
    }

    text
}

fn run(filepath: &str,
       actions: HashMap<rustbox::Key, viewer::Action>,
       show_line_num: bool)
       -> Result<()> {
    // Get file content and start a Viewer with it
    let text = open_file(filepath);
    let filename = match filepath.to_string().split('/').last() {
        Some(name) => name.to_string(),
        None => "unknown".to_string(),
    };

    let mut viewer =
        viewer::Viewer::new(&text, filename, actions, filepath, show_line_num);

    // Wait for keyboard events
    match viewer.poll_event() {
        Ok(_) => Ok(()),
        Err(error) => bail!(error),
    }
}
