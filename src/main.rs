// ENTE (Educational Nimble Text Editor)

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

use docopt::Docopt;
use slog::DrainExt;
use std::collections::HashMap;

use std::env;
use std::fs::File;
use std::io;

mod errors {
    error_chain!{}
}

use errors::*;

mod viewer;
mod model;
mod keyconfig;
mod colorconfig;

#[cfg(test)]
mod modeltest;

const FILE_POS_LOG: usize = 65;

struct LogFormat;

const USAGE: &'static str = "
Ente text editor.

Usage:
  ente FILE [--keyconfig=<kc>] \
     [--colorconfig=<cc>] [--hide-line-num] [--insert-tab-char] \
     [--tab-size=<ts>]
  ente (-h | --help)
  ente --version

  Options:
    -h --help             Show this screen.
    --version             Show version.
    --keyconfig=<kc>      Key configuration file.
    --colorconfig=<cc>    Color configuration file.
    --hide-line-num       Hide line numbers.
    --insert-tab-char     Insert a Tab instead of a number of spaces.
    --tab-size=<ts>       Size of tab in columns. (4 by default)
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_file: String,
    flag_keyconfig: String,
    flag_colorconfig: String,
    flag_hidelinenum: bool,
    flag_inserttabchar: bool,
    flag_tabsize: String,
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
        // No key config file set
        key_config_file = match env::home_dir() {
            Some(mut path) => {
                path.push(".config/ente/keys.conf");
                path.to_str().unwrap().to_string()
            }
            None => String::from("keys.conf"),
        }
    }

    let mut color_config_file = args.get_str("--colorconfig").to_string();
    if color_config_file.is_empty() {
        // No color config file set
        color_config_file = match env::home_dir() {
            Some(mut path) => {
                path.push(".config/ente/colors.conf");
                path.to_str().unwrap().to_string()
            }
            None => String::from("colors.conf"),
        }
    }

    let actions = keyconfig::fill_key_map(key_config_file.as_ref());
    let colors = colorconfig::fill_colors(color_config_file.as_ref());

    let hide_line_num = args.get_bool("--hide-line-num");
    let insert_tab_char = args.get_bool("--insert-tab-char");
    if hide_line_num {
        info!("Hide line numbers");
    }
    if insert_tab_char {
        info!("Insert tab characters");
    }

    let tab_size = match args.get_str("--tab-size").parse::<usize>() {
        Ok(i) => i,
        Err(_) => 0,
    };

    // Run catching errors
    if let Err(ref e) = run(args.get_str("FILE"),
                            actions,
                            colors,
                            !hide_line_num,
                            insert_tab_char,
                            tab_size) {
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
       actions: HashMap<rustbox::Key, viewer::Action>,
       colors: viewer::Colors,
       show_line_num: bool,
       insert_tab_char: bool,
       tab_size: usize)
       -> Result<()> {
    // Get file content and start a Viewer with it
    let filename = match filepath.to_string().split('/').last() {
        Some(name) => name.to_string(),
        None => "unknown".to_string(),
    };

    let mut viewer = viewer::Viewer::new(filepath,
                                         filename,
                                         actions,
                                         colors,
                                         show_line_num,
                                         insert_tab_char,
                                         tab_size);

    // Wait for keyboard events
    match viewer.poll_event() {
        Ok(_) => Ok(()),
        Err(error) => bail!(error),
    }
}
