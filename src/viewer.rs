extern crate rustbox;
extern crate slog_term;
extern crate time;
extern crate slog_stream;
extern crate slog_json;

use std::default::Default;

use rustbox::{Color, RustBox, OutputMode, EventResult};
use rustbox::Key;

mod errors {
    error_chain! { }
}

use errors::*;

pub struct Viewer {
    rustbox: RustBox,
    height: usize,    // window height without status line
    width: usize,
    filename: String,
    cur: usize
}

impl Viewer {
    pub fn new(filename: String) -> Viewer {
        let mut rustbox = RustBox::init(Default::default()).unwrap();
        let height = rustbox.height() - 1;
        let width = rustbox.width();
        rustbox.set_output_mode(OutputMode::EightBit);
        info!("Terminal window height: {}", height);

        Viewer {
            rustbox: rustbox,
            height: height,
            width: width,
            filename: filename,
            cur: 1,
        }
    }

    pub fn init(&mut self, text: &String, line_count: usize) {
        match self.display_chunk(&text, line_count, 1) {
            Ok(_) => self.update(),
            Err(_) => {
                self.rustbox.print(1, 1, rustbox::RB_NORMAL, Color::Red,
                                   Color::Black, "Empty file!");
                self.update()
            }
        }
    }

    pub fn display_chunk(&mut self, text: &String, line_count: usize,
                     start: usize) -> Result<()> {
        self.rustbox.clear();

        if start > line_count {
            warn!("Line {} past EOF", start);
            return Err("End of file".into());
        }

        self.cur = start;

        let mut lines = text.lines().skip(start - 1);
        for ln in 0 .. (self.height) {
            if let Some(line) = lines.next() {
                self.rustbox.print(1, ln, rustbox::RB_NORMAL, Color::White,
                                   Color::Black, line);
            } else {
                info!("Displayed range {} : {} lines", start,
                   start + ln - 1);
                return Ok(());
            }
        }

        info!("Displayed range {} : {} lines", start, start + self.height);
        Ok(())
    }

    pub fn scroll(&mut self, text: &String, line_count: usize, key: rustbox::Key) {
        let mut cur = self.cur;
        match key {
            Key::Down => {
                // Scroll by one until last line is in the bottom of the window
                if cur <= line_count - self.height {
                    cur += 1;
                }
                match self.display_chunk(&text, line_count, cur) {
                    Ok(_) => self.update(),
                    Err(_) => {}
                }
            }
            Key::Up => {
                // Scroll by one to the top of the file
                if cur > 1 {
                    cur -= 1;
                }
                match self.display_chunk(&text, line_count, cur) {
                    Ok(_) => self.update(),
                    Err(_) => {}
                }
            }
            Key::PageDown => {
                // Scroll a window height down
                if cur <= line_count - (self.height * 2) {
                    cur += self.height;
                } else {
                    cur = line_count - self.height + 1;
                }
                match self.display_chunk(&text, line_count, cur) {
                    Ok(_) => self.update(),
                    Err(_) => {}
                }
            }
            Key::PageUp => {
                // Scroll a window height up
                if cur > self.height {
                    cur -= self.height;
                } else {
                    cur = 1;
                }
                match self.display_chunk(&text, line_count, cur) {
                    Ok(_) => self.update(),
                    Err(_) => {}
                }
            }
            _ => {}
        }
    }

    pub fn poll_event(&mut self) -> EventResult {
        self.rustbox.poll_event(false)
    }

    fn update(&mut self) {
        // Add an informational status line
        let filestatus = format!("{} ({})", self.filename, self.cur);

        let help: &'static str = "Press 'q' to quit";
        self.rustbox.print(1, self.height, rustbox::RB_REVERSE, Color::White,
                           Color::Black, filestatus.as_ref());
        self.rustbox.print(self.width - help.len(), self.height,
                           rustbox::RB_REVERSE, Color::White,
                           Color::Black, help);

        self.rustbox.present();
    }
}
