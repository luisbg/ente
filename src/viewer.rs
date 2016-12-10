extern crate rustbox;
extern crate slog_term;
extern crate time;
extern crate slog_stream;
extern crate slog_json;

use std::default::Default;

use rustbox::{Color, RustBox, OutputMode};
use rustbox::Key;

mod errors {
    error_chain!{}
}

use errors::*;

const RB_COL_START: usize = 0;
const RB_ROW_START: usize = 0;

enum Action {
    None,
    MoveRight,
    MoveLeft,
    MoveDown,
    MoveUp,
    MovePageUp,
    MovePageDown,
    MoveStartLine,
    MoveEndLine,
}

pub struct Cursor {
    line: usize,
    col: usize,
}

pub struct Viewer {
    rustbox: RustBox,
    text: String,
    height: usize, // window height without status line
    width: usize,
    filename: String,
    disp_line: usize, // first displayed line
    disp_col: usize, // first displayed col
    focus_col: usize,
    cur_line_len: usize,
    line_count: usize,
    cursor: Cursor,
}

impl Viewer {
    pub fn new(text: &String, filename: String) -> Viewer {
        let mut rustbox = RustBox::init(Default::default()).unwrap();
        let height = rustbox.height() - 1;
        let width = rustbox.width();
        rustbox.set_output_mode(OutputMode::EightBit);
        info!("Terminal window height: {}", height);

        rustbox.set_cursor(0, 0);

        let text_copy = text.clone();
        let cursor = Cursor { line: 1, col: 1 };
        let line_count = text.lines().count();

        let mut view = Viewer {
            rustbox: rustbox,
            text: text_copy,
            height: height,
            width: width,
            filename: filename,
            disp_line: 1,
            disp_col: 1,
            focus_col: 1,
            cur_line_len: 1,
            line_count: line_count,
            cursor: cursor,
        };

        view.set_current_line(1);
        match view.display_chunk(1, 1) {
            Ok(_) => view.update(),
            Err(_) => {
                view.rustbox.print(RB_COL_START,
                                   RB_ROW_START,
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
                         start_line: usize,
                         start_col: usize)
                         -> Result<()> {
        self.rustbox.clear();

        if start_line > self.line_count {
            warn!("Line {} past EOF", start_line);
            return Err("End of file".into());
        }

        self.disp_line = start_line;
        self.disp_col = start_col;

        let mut lines = self.text.lines().skip(start_line - 1);
        for ln in 0..(self.height) {
            if let Some(line) = lines.next() {
                if line.len() >= start_col {
                    self.rustbox.print(RB_COL_START,
                                       ln,
                                       rustbox::RB_NORMAL,
                                       Color::White,
                                       Color::Black,
                                       &line[start_col - 1..]);
                } else {
                    self.rustbox.print(RB_COL_START,
                                       ln,
                                       rustbox::RB_NORMAL,
                                       Color::White,
                                       Color::Black,
                                       "");
                }
            } else {
                info!("Displayed range {} : {} lines",
                      start_line,
                      start_line + ln - 1);
                return Ok(());
            }
        }

        info!("Displayed range {} : {} lines",
              start_line,
              start_line + self.height);
        Ok(())
    }

    pub fn scroll(&mut self, action: &Action) {
        let mut disp_line = self.disp_line;
        let disp_col = self.disp_col;

        match *action {
            Action::MoveDown => {
                // Scroll by one until last line is in the bottom of the window
                if disp_line <= self.line_count - self.height {
                    disp_line += 1;
                }
                match self.display_chunk(disp_line, disp_col) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
            Action::MoveUp => {
                // Scroll by one to the top of the file
                if disp_line > 1 {
                    disp_line -= 1;
                }
                match self.display_chunk(disp_line, disp_col) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
            Action::MovePageDown => {
                if self.line_count < self.height {
                    warn!("Can't scroll files smaller than the window");
                    return;
                }

                // Scroll a window height down
                if disp_line <= self.line_count - self.height &&
                   disp_line + self.height <= self.line_count - self.height {
                    disp_line += self.height;
                } else {
                    disp_line = self.line_count - self.height + 1;
                }

                match self.display_chunk(disp_line, disp_col) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
            Action::MovePageUp => {
                // Scroll a window height up
                if disp_line > self.height {
                    disp_line -= self.height;
                } else {
                    disp_line = 1;
                }
                match self.display_chunk(disp_line, disp_col) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
            Action::MoveLeft => {
                let disp_col = self.disp_col - 1;
                let disp_line = self.disp_line;
                match self.display_chunk(disp_line, disp_col) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
            Action::MoveRight => {
                let disp_col = self.disp_col + 1;
                let disp_line = self.disp_line;
                match self.display_chunk(disp_line, disp_col) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
            _ => {}
        }
    }

    pub fn move_cursor(&mut self, action: &Action) {
        match *action {
            Action::MoveDown => {
                if self.cursor.line < self.line_count {
                    let tmp = self.cursor.line + 1;
                    self.set_current_line(tmp);
                    info!("Current line is {}", self.cursor.line);

                    if self.cursor.line + 1 > (self.disp_line + self.height) {
                        self.scroll(action);
                    }
                } else {
                    info!("Can't go down, already at the bottom of file");
                    return;
                }
            }
            Action::MoveUp => {
                if self.cursor.line > 1 {
                    let tmp = self.cursor.line - 1;
                    self.set_current_line(tmp);
                    info!("Current line is {}", self.cursor.line);

                    if self.cursor.line < self.disp_line {
                        self.scroll(action);
                    }
                } else {
                    info!("Can't go up, already at the top of file");
                    return;
                }
            }
            Action::MoveLeft => {
                if self.cursor.col > 1 {
                    self.cursor.col -= 1;
                    self.focus_col = self.cursor.col;

                    if self.cursor.col < self.disp_col {
                        self.scroll(action);
                    }
                } else {
                    info!("Can't go left, already at beginning of the line");
                    return;
                }
            }
            Action::MoveRight => {
                if self.focus_col < self.cur_line_len {
                    self.cursor.col += 1;
                    self.focus_col = self.cursor.col;

                    if self.focus_col > self.disp_col + self.width - 1 {
                        self.scroll(action);
                    }
                } else {
                    info!("Can't go right, already at end of the line");
                    return;
                }
            }
            Action::MovePageDown => {
                if self.cursor.line + self.height < self.line_count {
                    let tmp = self.cursor.line + self.height;
                    self.set_current_line(tmp);
                } else {
                    let line_count = self.line_count;
                    self.set_current_line(line_count);
                }

                self.scroll(action);
            }
            Action::MovePageUp => {
                if self.cursor.line > self.height {
                    let tmp = self.cursor.line - self.height;
                    self.set_current_line(tmp);
                } else {
                    self.set_current_line(1);
                }

                self.scroll(action);
            }
            Action::MoveStartLine => {
                if self.cur_line_len > 0 {
                    self.cursor.col = 1;
                    self.focus_col = 1;
                } else {
                    info!("Can't move to the beginning of an empty line");
                }
            }
            Action::MoveEndLine => {
                if self.cur_line_len > 0 {
                    self.cursor.col = self.cur_line_len;
                    self.focus_col = self.cur_line_len;
                } else {
                    info!("Can't move to the end of an empty line");
                }
            }
            Action::None => {}
        }

        match *action {
            Action::MoveDown | Action::MoveUp | Action::MovePageDown |
            Action::MovePageUp => {
                let tmp_cur_col: usize;
                if self.cursor.col == 0 {
                    tmp_cur_col = 1;
                } else {
                    tmp_cur_col = self.cursor.col;
                }
                if tmp_cur_col < self.disp_col {
                    // Cursor before display, scroll left
                    let disp_col = tmp_cur_col;
                    let disp_line = self.disp_line;
                    match self.display_chunk(disp_line, disp_col) {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }

                if self.cursor.col > self.disp_col + self.width {
                    // Cursor past display, scroll right
                    let disp_col = self.cursor.col - self.width;
                    let disp_line = self.disp_line;
                    match self.display_chunk(disp_line, disp_col) {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }

            }
            _ => {}
        }

        self.update();
    }

    pub fn poll_event(&mut self) -> Result<()> {
        loop {
            match self.rustbox.poll_event(false) {
                Ok(rustbox::Event::KeyEvent(key)) => {
                    // TODO: Handle quit action better
                    if !self.match_key_action(key) {
                        return Ok(());
                    }
                }
                Err(_) => {
                    let e = "Rustbox.poll_event Error";
                    error!(e);
                    return Err(e.into());
                }
                _ => {}
            }
        }
    }

    fn match_key_action(&mut self, key: Key) -> bool {
        let mut action: Action = Action::None;
        match key {
            Key::Char('q') => {
                info!("Quitting application");
                return false;
            }
            Key::Up => {
                action = Action::MoveUp;
            }
            Key::Down => {
                action = Action::MoveDown;
            }
            Key::Left => {
                action = Action::MoveLeft;
            }
            Key::Right => {
                action = Action::MoveRight;
            }
            Key::PageDown => {
                action = Action::MovePageDown;
            }
            Key::PageUp => {
                action = Action::MovePageUp;
            }
            Key::Home => {
                action = Action::MoveStartLine;
            }
            Key::End => {
                action = Action::MoveEndLine;
            }
            _ => {}
        }

        match action {
            Action::MoveUp | Action::MoveDown | Action::MoveLeft |
            Action::MoveRight | Action::MovePageDown | Action::MovePageUp |
            Action::MoveStartLine | Action::MoveEndLine => {
                self.move_cursor(&action);
            }
            _ => {}
        }

        true
    }


    fn set_current_line(&mut self, line_num: usize) {
        self.cursor.line = line_num;

        let line = match self.text.lines().nth(self.cursor.line - 1) {
            Some(line) => line,
            None => return,
        };
        self.cur_line_len = line.len();

        if self.cur_line_len < self.focus_col {
            // previous line was longer
            self.cursor.col = self.cur_line_len;
        } else {
            self.cursor.col = self.focus_col;

            if self.cursor.col == 0 {
                // previous line was empty
                self.cursor.col = 1;   // jump back to first column
            }
        }
    }

    fn update(&mut self) {
        // Add an informational status line
        let filestatus = format!("{} ({},{})",
                                 self.filename,
                                 self.cursor.line,
                                 self.cursor.col);
        let cur_col: isize;
        if self.cursor.col == 0 {
            cur_col = 0;
        } else {
            cur_col = (self.cursor.col - self.disp_col) as isize;
        }
        self.rustbox.set_cursor(cur_col,
                                (self.cursor.line - self.disp_line) as isize);

        let help: &'static str = "Press 'q' to quit";
        self.rustbox.print(RB_COL_START,
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
