extern crate rustbox;
extern crate slog_term;
extern crate time;
extern crate slog_stream;
extern crate slog_json;

use std::default::Default;

use rustbox::{Color, RustBox, OutputMode, EventResult};
use rustbox::Key;

mod errors {
    error_chain!{}
}

use errors::*;

pub struct Viewer {
    rustbox: RustBox,
    height: usize, // window height without status line
    width: usize,
    filename: String,
    disp_line: usize, // first displayed line
    cur_line: usize,
    cur_col: usize,
    saved_cur_col: usize,
    cur_line_len: usize,
}

impl Viewer {
    pub fn new(text: &String, filename: String, line_count: usize) -> Viewer {
        let mut rustbox = RustBox::init(Default::default()).unwrap();
        let height = rustbox.height() - 1;
        let width = rustbox.width();
        rustbox.set_output_mode(OutputMode::EightBit);
        info!("Terminal window height: {}", height);

        rustbox.set_cursor(0, 0);

        let mut view = Viewer {
            rustbox: rustbox,
            height: height,
            width: width,
            filename: filename,
            disp_line: 1,
            cur_line: 1,
            cur_col: 1,
            saved_cur_col: 1,
            cur_line_len: 1,
        };

        view.set_current_line(&text, 1);
        match view.display_chunk(&text, line_count, 1) {
            Ok(_) => view.update(),
            Err(_) => {
                view.rustbox.print(0,
                                   0,
                                   rustbox::RB_NORMAL,
                                   Color::Red,
                                   Color::Black,
                                   "Empty file!");
                view.disp_line = 0;
                view.update()
            }
        }

        return view;
    }

    pub fn display_chunk(&mut self,
                         text: &String,
                         line_count: usize,
                         start: usize)
                         -> Result<()> {
        self.rustbox.clear();

        if start > line_count {
            warn!("Line {} past EOF", start);
            return Err("End of file".into());
        }

        self.disp_line = start;

        let mut lines = text.lines().skip(start - 1);
        for ln in 0..(self.height) {
            if let Some(line) = lines.next() {
                self.rustbox.print(0,
                                   ln,
                                   rustbox::RB_NORMAL,
                                   Color::White,
                                   Color::Black,
                                   line);
            } else {
                info!("Displayed range {} : {} lines",
                      start,
                      start + ln - 1);
                return Ok(());
            }
        }

        info!("Displayed range {} : {} lines",
              start,
              start + self.height);
        Ok(())
    }

    pub fn scroll(&mut self,
                  text: &String,
                  line_count: usize,
                  key: rustbox::Key) {
        if line_count < self.height {
            warn!("Can't scroll files smaller than the window");
            return;
        }

        let mut disp_line = self.disp_line;
        match key {
            Key::Down => {
                // Scroll by one until last line is in the bottom of the window
                if disp_line <= line_count - self.height {
                    disp_line += 1;
                }
                match self.display_chunk(&text, line_count, disp_line) {
                    Ok(_) => self.update(),
                    Err(_) => {}
                }
            }
            Key::Up => {
                // Scroll by one to the top of the file
                if disp_line > 1 {
                    disp_line -= 1;
                }
                match self.display_chunk(&text, line_count, disp_line) {
                    Ok(_) => self.update(),
                    Err(_) => {}
                }
            }
            Key::PageDown => {
                // Scroll a window height down
                if disp_line <= line_count - (self.height * 2) {
                    disp_line += self.height;
                } else {
                    disp_line = line_count - self.height + 1;
                }
                match self.display_chunk(&text, line_count, disp_line) {
                    Ok(_) => {
                        if self.cur_line + self.height < line_count {
                            let tmp = self.cur_line + self.height;
                            self.set_current_line(text, tmp);
                        } else {
                            self.set_current_line(text, line_count);
                        }

                        self.update();
                    }
                    Err(_) => {}
                }
            }
            Key::PageUp => {
                // Scroll a window height up
                if disp_line > self.height {
                    disp_line -= self.height;
                } else {
                    disp_line = 1;
                }
                match self.display_chunk(&text, line_count, disp_line) {
                    Ok(_) => {
                        if self.cur_line > self.height {
                            let tmp = self.cur_line - self.height;
                            self.set_current_line(text, tmp);
                        } else {
                            self.set_current_line(text, 1);
                        }

                        self.update();
                    }
                    Err(_) => {}
                }
            }
            _ => {}
        }
    }

    pub fn move_cursor(&mut self,
                       text: &String,
                       line_count: usize,
                       key: rustbox::Key) {
        match key {
            Key::Down => {

                if self.cur_line < line_count {
                    let tmp = self.cur_line + 1;
                    self.set_current_line(text, tmp);
                    info!("Current line is {}", self.cur_line);

                    if self.cur_line + 1 > (self.disp_line + self.height) {
                        self.scroll(text, line_count, key);
                    }

                    self.update();
                } else {
                    info!("Can't go down, already at the bottom of file");
                }
            }
            Key::Up => {
                if self.cur_line > 1 {
                    let tmp = self.cur_line - 1;
                    self.set_current_line(text, tmp);
                    info!("Current line is {}", self.cur_line);

                    if self.cur_line < self.disp_line {
                        self.scroll(text, line_count, key);
                    }

                    self.update();
                } else {
                    info!("Can't go up, already at the top of file");
                }
            }
            Key::Left => {
                if self.saved_cur_col > 1 {
                    self.saved_cur_col -= 1;
                    self.cur_col -= 1;
                    self.update();
                } else {
                    info!("Can't go left, already at beginning of line");
                }
            }
            Key::Right => {
                if self.saved_cur_col < self.cur_line_len {
                    self.saved_cur_col += 1;
                    self.cur_col += 1;
                    self.update();
                }
            }
            _ => {}
        }
    }

    pub fn poll_event(&mut self) -> EventResult {
        self.rustbox.poll_event(false)
    }

    fn set_current_line(&mut self, text: &String, cur_line: usize) {
        self.cur_line = cur_line;

        let line = match text.lines().nth(self.cur_line - 1) {
            Some(line) => line,
            None => return
        };
        self.cur_line_len = line.len();

        if self.cur_line_len < self.saved_cur_col {  // previous line was longer
                self.cur_col = self.cur_line_len;
        } else {
            self.cur_col = self.saved_cur_col;

            if self.cur_col == 0 {  // previous line was empty
                self.cur_col = 1;   // jump back to first column
            }
        }
    }

    fn update(&mut self) {
        // Add an informational status line
        let filestatus = format!("{} ({},{})",
                                 self.filename,
                                 self.cur_line,
                                 self.cur_col);

        let cur_col: isize;
        if self.cur_col == 0 {
            cur_col = 0;
        } else {
            cur_col = (self.cur_col - 1) as isize;
        }
        self.rustbox.set_cursor(cur_col,
                                (self.cur_line - self.disp_line) as isize);

        let help: &'static str = "Press 'q' to quit";
        self.rustbox.print(0,
                           self.height,
                           rustbox::RB_REVERSE,
                           Color::White,
                           Color::Black,
                           filestatus.as_ref());
        self.rustbox.print(self.width - help.len(),
                           self.height,
                           rustbox::RB_REVERSE,
                           Color::White,
                           Color::Black,
                           help);

        self.rustbox.present();
    }
}
