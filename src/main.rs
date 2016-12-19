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
extern crate rustc_serialize;
extern crate docopt;

use std::error::Error as StdError;
use std::fs::File;
use std::io::prelude::*;
use std::io;

use std::env;
use slog::DrainExt;
use std::collections::HashMap;
use docopt::Docopt;

mod errors {
    error_chain!{}
}

use errors::*;

mod viewer;
mod keyconfig;

const FILE_POS_LOG: usize = 65;

struct LogFormat;

const USAGE: &'static str = "
Ente text editor.

Usage:
  ente FILE [--keyconfig=<kc>]
  ente (-h | --help)
  ente --version

  Options:
    -h --help          Show this screen.
    --version          Show version.
    --keyconfig=<kc>   Key configurationf file.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_file: String,
    flag_keyconfig: String,
}

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
    if key_config_file.len() == 0 {
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

    // Run catching errors
    if let Err(ref e) = run(args.get_str("FILE"), actions) {
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

fn run(filepath: &str,
       actions: HashMap<rustbox::Key, viewer::Action>)
       -> Result<()> {
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

    let text_clone = text.clone();
    let mut viewer = viewer::Viewer::new(&text_clone, filename, actions);

    // Wait for keyboard events
    match viewer.poll_event() {
        Ok(_) => Ok(()),
        Err(error) => bail!(error),
    }
}
