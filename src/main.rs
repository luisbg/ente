// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate rustbox;
#[macro_use] extern crate error_chain;

use std::error::{Error as StdError};
use std::default::Default;
use std::fs::File;
use std::io::prelude::*;

use rustbox::{Color, RustBox, OutputMode};
use rustbox::Key;

mod errors {
    error_chain! { }
}

use errors::*;

fn main() {
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
    let mut rustbox = match RustBox::init(Default::default()) {
        Ok(rustbox) => rustbox,
        Err(e) => bail!("{}", e),
    };

    rustbox.set_output_mode(OutputMode::EightBit);

    let mut file = File::open("/tmp/kerouac")
          .chain_err(|| "Couldn't open file")?;
    let mut text = String::new();
    match file.read_to_string(&mut text) {
        Err(why) => bail!("couldn't read {}: ", why.description()),
        Ok(_) => {},
    }
    let mut lines = text.lines();
    if let Some(line) = lines.next() {
        rustbox.print(1, 1, rustbox::RB_BOLD, Color::White, Color::Black,
                      line);
    }
    rustbox.print(1, 3, rustbox::RB_NORMAL, Color::Black, Color::Byte(0x04),
                  "Press 'q' to quit.");

    rustbox.present();
    loop {
        match rustbox.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Key::Char('q') => { break; }
                    _ => { }
                }
            },
            Err(e) => bail!("Rustbox.poll_event Error {}", e.description()),
            _ => { }
        }
    }

    Ok(())
}
