extern crate rustbox;

use std::error::Error;
use std::default::Default;

use rustbox::{Color, RustBox, OutputMode};
use rustbox::Key;

fn main() {
    let mut rustbox = match RustBox::init(Default::default()) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
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
            Err(e) => panic!("{}", e.description()),
            _ => { }
        }
    }
}
