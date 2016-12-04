// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate rustbox;
#[macro_use] extern crate error_chain;

use std::error::{Error as StdError};
use std::default::Default;

use rustbox::{Color, RustBox, OutputMode};
use rustbox::Key;

mod errors {
    error_chain! { }
}

use errors::Result;

fn main() {
    if let Err(ref e) = run() {
        println!("error: {}", e);
            for e in e.iter().skip(1) {
                println!("caused by: {}", e);
            }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}",
                     backtrace);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut rustbox = match RustBox::init(Default::default()) {
        Ok(v) => v,
        Err(e) => bail!("{}", e),
    };

    rustbox.set_output_mode(OutputMode::EightBit);

    rustbox.print(1, 1, rustbox::RB_BOLD, Color::White, Color::Black,
                  "Hello, world!");
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
