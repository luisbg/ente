extern crate rustbox;
extern crate time;
extern crate slog_stream;

use std::default::Default;
use std::collections::HashMap;

use rustbox::{Color, RustBox, OutputMode};
use rustbox::Key;

use model;

mod errors {}

use errors::*;

const RB_COL_START: usize = 0;
const RB_ROW_START: usize = 0;

#[derive(Copy,Clone)]
pub enum Action {
    None,
    Go,
    MoveRight,
    MoveLeft,
    MoveDown,
    MoveUp,
    MovePageUp,
    MovePageDown,
    MoveStartLine,
    MoveEndLine,
    GoToLine,
    Search,
    SearchNext,
    SearchPrevious,
    MoveNextWord,
    MovePrevWord,
    EditMode,
    ReadMode,
    Save,
    Quit,
}

#[derive(Eq,PartialEq)]
enum Mode {
    Read,
    Edit,
    GoToLine,
    Search,
}

pub struct Cursor {
    line: usize,
    col: usize,
}

pub struct Viewer {
    rustbox: RustBox,
    text: String,
    model: model::Model,
    mode: Mode,
    actions: HashMap<Key, Action>,
    height: usize, // window height without status line
    width: usize,
    show_line_num: bool,
    filename: String,
    disp_line: usize, // first displayed line
    disp_col: usize, // first displayed col
    focus_col: usize,
    cur_line_len: usize,
    line_jump: usize,
    cursor: Cursor,
    search_string: String,
}

fn number_of_digits(number: usize) -> usize {
    let mut tmp = number;
    let mut digits: usize = 0;
    while tmp > 0 {
        tmp /= 10;
        digits += 1;
    }

    digits
}

impl Viewer {
    pub fn new(text: &str,
               filename: String,
               key_map: HashMap<Key, Action>,
               filepath: &str,
               show_line_num: bool)
               -> Viewer {
        let mut rustbox = RustBox::init(Default::default()).unwrap();
        let height = rustbox.height() - 1;
        rustbox.set_output_mode(OutputMode::EightBit);
        info!("Terminal window height: {}", height);

        rustbox.set_cursor(0, 0);

        let cursor = Cursor { line: 1, col: 1 };
        let mut model = model::Model::new(text, filepath);

        let width = if show_line_num {
            rustbox.width() - number_of_digits(model.get_line_count()) - 1
        } else {
            rustbox.width()
        };

        let mut view = Viewer {
            rustbox: rustbox,
            text: String::from(text),
            model: model,
            mode: Mode::Read,
            actions: key_map,
            height: height,
            width: width,
            show_line_num: show_line_num,
            filename: filename,
            disp_line: 1,
            disp_col: 1,
            focus_col: 1,
            cur_line_len: 1,
            line_jump: 0,
            cursor: cursor,
            search_string: String::new(),
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

        view
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

    fn display_chunk(&mut self,
                     start_line: usize,
                     start_col: usize)
                     -> Result<()> {
        self.rustbox.clear();

        if start_line > self.model.get_line_count() {
            warn!("Line {} past EOF", start_line);
            return Err("End of file".into());
        }

        self.disp_line = start_line;
        self.disp_col = start_col;

        let mut lines = self.text.lines().skip(start_line - 1);
        for ln in 0..(self.height) {
            if let Some(line) = lines.next() {
                let beg = start_col - 1;

                // Check if there is line content to show or past the end
                if line.len() > beg {
                    let end = if (line.len() - beg) >= self.width {
                        // Don't show characters past terminal's right edge
                        beg + self.width
                    } else {
                        line.len()
                    };
                    self.rustbox.print(RB_COL_START,
                                       ln,
                                       rustbox::RB_NORMAL,
                                       Color::White,
                                       Color::Black,
                                       &line[beg..end]);
                } else {
                    self.rustbox.print(RB_COL_START,
                                       ln,
                                       rustbox::RB_NORMAL,
                                       Color::White,
                                       Color::Black,
                                       "");
                }
                if self.show_line_num {
                    self.rustbox.print(RB_COL_START + self.width + 1,
                                       ln,
                                       rustbox::RB_NORMAL,
                                       Color::Blue,
                                       Color::Black,
                                       format!("{}", ln + start_line).as_ref());
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

    fn vertical_scroll(&mut self, action: Action) {
        let mut disp_line = self.disp_line;
        let mut disp_col = self.disp_col;
        let line_count = self.model.get_line_count();

        match action {
            Action::MoveDown => {
                // Scroll by one until last line is in the bottom of the window
                if disp_line <= line_count - self.height {
                    disp_line += 1;
                }
            }
            Action::MoveUp => {
                // Scroll by one to the top of the file
                if disp_line > 1 {
                    disp_line -= 1;
                }
            }
            Action::MovePageDown => {
                if line_count < self.height {
                    warn!("Can't scroll files smaller than the window");
                    return;
                }

                // Scroll a window height down
                if disp_line <= line_count - self.height &&
                   disp_line + self.height <= line_count - self.height {
                    disp_line += self.height;
                } else {
                    disp_line = line_count - self.height + 1;
                }
            }
            Action::MovePageUp => {
                // Scroll a window height up
                if disp_line > self.height {
                    disp_line -= self.height;
                } else {
                    disp_line = 1;
                }
            }
            Action::MoveLeft => {
                disp_col = self.disp_col - 1;
            }
            Action::MoveRight => {
                disp_col = self.disp_col + 1;
            }
            Action::MoveStartLine => {
                disp_col = 1;
            }
            Action::MoveEndLine => {
                disp_col = self.cursor.col - self.width + 1;
            }
            _ => {
                return;
            }
        }
        let _ = self.display_chunk(disp_line, disp_col);
    }

    fn horizontal_scroll(&mut self, action: Action) {
        match action {
            Action::MoveDown | Action::MoveUp | Action::MovePageDown |
            Action::MovePageUp => {
                let tmp_cur_col = if self.cursor.col == 0 {
                    1
                } else {
                    self.cursor.col
                };
                if tmp_cur_col < self.disp_col {
                    // Cursor before display, scroll left
                    let disp_col = tmp_cur_col;
                    let disp_line = self.disp_line;
                    let _ = self.display_chunk(disp_line, disp_col);
                }

                if self.cursor.col > self.disp_col + self.width - 1 {
                    // Cursor past display, scroll right
                    let disp_col = self.cursor.col - self.width + 1;
                    let disp_line = self.disp_line;
                    let _ = self.display_chunk(disp_line, disp_col);
                }

            }
            _ => {}
        }

    }

    fn move_cursor(&mut self, action: Action) {
        let line_count = self.model.get_line_count();

        match action {
            Action::MoveDown => {
                if self.cursor.line < line_count {
                    let tmp = self.cursor.line + 1;
                    self.set_current_line(tmp);
                    info!("Current line is {}", self.cursor.line);

                    if self.cursor.line + 1 > (self.disp_line + self.height) {
                        self.vertical_scroll(action);
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
                        self.vertical_scroll(action);
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
                        self.vertical_scroll(action);
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

                    if self.cursor.col > self.disp_col + self.width - 1 {
                        self.vertical_scroll(action);
                    }
                } else {
                    info!("Can't go right, already at end of the line");
                    return;
                }
            }
            Action::MovePageDown => {
                if self.cursor.line + self.height < line_count {
                    let tmp = self.cursor.line + self.height;
                    self.set_current_line(tmp);
                } else {
                    self.set_current_line(line_count);
                }

                self.vertical_scroll(action);
            }
            Action::MovePageUp => {
                if self.cursor.line > self.height {
                    let tmp = self.cursor.line - self.height;
                    self.set_current_line(tmp);
                } else {
                    self.set_current_line(1);
                }

                self.vertical_scroll(action);
            }
            Action::MoveStartLine => {
                if self.cur_line_len > 0 {
                    self.cursor.col = 1;
                    self.focus_col = 1;
                } else {
                    info!("Can't move to the beginning of an empty line");
                }

                if self.cursor.col < self.disp_col {
                    self.vertical_scroll(action);
                }
            }
            Action::MoveEndLine => {
                if self.cur_line_len > 0 {
                    self.cursor.col = self.cur_line_len;
                    self.focus_col = self.cur_line_len;
                } else {
                    info!("Can't move to the end of an empty line");
                }

                if self.cursor.col > self.disp_col + self.width - 1 {
                    self.vertical_scroll(action);
                }
            }
            _ => {}
        }

        // Depending on differences in lines we might have to scroll
        // horizontally
        self.horizontal_scroll(action);

        self.update();
    }

    fn match_key_action(&mut self, key: Key) -> bool {
        let no_action = Action::None;
        let action = *self.actions.get(&key).unwrap_or(&no_action);

        match self.mode {
            Mode::Edit => {
                match action {
                    Action::Quit => {
                        info!("Quitting application");
                        return false;
                    }
                    Action::MoveUp | Action::MoveDown | Action::MoveLeft |
                    Action::MoveRight | Action::MovePageDown |
                    Action::MovePageUp | Action::MoveStartLine |
                    Action::MoveEndLine => {
                        self.move_cursor(action);
                    }
                    Action::ReadMode => {
                        self.switch_mode(action);

                        // Update current line max column
                        let cur_line = self.cursor.line;
                        self.set_current_line(cur_line);

                        self.update();
                    }
                    Action::Save => {
                        self.model.save();
                        self.update();
                    }
                    _ => {
                        match key {
                            Key::Char(c) => {
                                self.add_char(c);
                            }
                            Key::Enter => {
                                self.add_char('\n');
                            }
                            Key::Backspace => {
                                self.delete_char();
                            }
                            _ => {}
                        }
                    }
                }
            }
            Mode::Read => {
                match action {
                    Action::Quit => {
                        info!("Quitting application");
                        return false;
                    }
                    Action::MoveUp | Action::MoveDown | Action::MoveLeft |
                    Action::MoveRight | Action::MovePageDown |
                    Action::MovePageUp | Action::MoveStartLine |
                    Action::MoveEndLine => {
                        self.move_cursor(action);
                    }
                    Action::GoToLine => {
                        info!("Enter GoToLine mode");
                        self.mode = Mode::GoToLine;
                        self.update();
                    }
                    Action::Search => {
                        info!("Enter Search mode");
                        self.mode = Mode::Search;
                        self.search_string = String::new();
                        self.update();
                    }
                    Action::SearchNext => {
                        self.do_forward_search();
                    }
                    Action::SearchPrevious => {
                        self.do_backward_search();
                    }
                    Action::MoveNextWord => {
                        self.move_next_word();
                    }
                    Action::MovePrevWord => {
                        self.move_prev_word();
                    }
                    Action::EditMode => {
                        self.switch_mode(action);

                        // Update current line max column
                        let cur_line = self.cursor.line;
                        self.set_current_line(cur_line);

                        self.update();
                    }
                    Action::Save => {
                        self.model.save();
                        self.update();
                    }
                    _ => {}
                }
            }
            Mode::GoToLine => {
                match action {
                    Action::Go => {
                        self.do_line_jump();
                    }
                    _ => {
                        // Numbers don't always match GoToLine action
                        self.go_to_line_mode(key);
                    }
                }
            }
            Mode::Search => {
                match action {
                    Action::Go => {
                        self.do_forward_search();
                    }
                    Action::Quit => {
                        self.mode = Mode::Read;
                        self.update();
                    }
                    _ => {
                        self.search_mode(key);
                    }
                }
            }
        }

        true
    }

    fn switch_mode(&mut self, action: Action) {
        match action {
            Action::EditMode => {
                if self.mode != Mode::Edit {
                    info!("Switch to Edit Mode");
                    self.mode = Mode::Edit;
                }
            }
            Action::ReadMode => {
                if self.mode != Mode::Read {
                    info!("Switch to Read Mode");
                    self.mode = Mode::Read;
                }
            }
            _ => {}
        }
    }

    fn go_to_line_mode(&mut self, key: Key) {
        let n = match key {
            Key::Char('1') => 1,
            Key::Char('2') => 2,
            Key::Char('3') => 3,
            Key::Char('4') => 4,
            Key::Char('5') => 5,
            Key::Char('6') => 6,
            Key::Char('7') => 7,
            Key::Char('8') => 8,
            Key::Char('9') => 9,
            Key::Char('0') => 0,
            _ => {
                return;
            }
        };

        self.line_jump = (self.line_jump * 10) + n;
        self.update();
    }

    fn do_line_jump(&mut self) {
        let line_num = self.line_jump;

        self.mode = Mode::Read;  // Set back to previous mode
        self.line_jump = 0;

        if line_num > self.model.get_line_count() || line_num == 0 {
            info!("ERROR: Invalid line number {}", line_num);
            self.update();

            return;
        }

        info!("Go to line {}", line_num);
        self.set_cursor(line_num, 1);
        self.update();
    }

    fn search_mode(&mut self, key: Key) {
        match key {
            Key::Char(c) => {
                self.search_string.push(c);
            }
            Key::Backspace => {
                self.search_string.pop();
            }
            _ => return,
        }

        self.update();
    }

    fn do_forward_search(&mut self) {
        self.mode = Mode::Read;

        if self.search_string == "" {
            self.update();
            return;
        }

        info!("Search for next: {}", self.search_string);
        let text_copy = self.text.clone();  // so we can borrow self as mutable
        let mut lines = text_copy.lines().skip(self.cursor.line - 1);
        let mut line_num = 0;
        let mut col = 0;

        // Check current line after the cursor
        let (_, rest_line) = lines.next().unwrap().split_at(self.cursor.col);
        match rest_line.find(self.search_string.as_str()) {
            Some(c) => {
                line_num = self.cursor.line;
                col = c + self.cursor.col + 1;
            }
            None => {
                // If nothing found in current line, search in the rest
                for ln in self.cursor.line..self.model.get_line_count() {
                    match lines.next() {
                        Some(l) => {
                            if let Some(c) =
                                l.find(self.search_string.as_str()) {
                                line_num = ln + 1;
                                col = c + 1;
                                break;  // Found it
                            }
                        }
                        _ => {
                            return;
                        }
                    }
                }
            }
        }

        if line_num != 0 {
            info!("Found '{}' in line {}",
                  self.search_string,
                  line_num);
            self.set_cursor(line_num, col);
        } else {
            info!("Did not found: {}", self.search_string);
        }

        self.update();
    }

    fn do_backward_search(&mut self) {
        self.mode = Mode::Read;

        if self.search_string == "" {
            self.update();
            return;
        }

        info!("Search for previous: {}", self.search_string);
        let text_copy = self.text.clone();  // so we can borrow self as mutable
        let mut lines = text_copy.lines()
            .rev()
            .skip(self.model.get_line_count() - self.cursor.line);
        let mut line_num = 0;
        let mut col = 0;

        // Check current line before the cursor
        let (beg_line, _) = lines.next().unwrap().split_at(self.cursor.col);
        match beg_line.rfind(self.search_string.as_str()) {
            Some(c) => {
                line_num = self.cursor.line;
                col = c + 1;
            }
            None => {
                // If nothing found in current line, search in the rest
                for ln in (1..self.cursor.line).rev() {
                    match lines.next() {
                        Some(l) => {
                            if let Some(c) =
                                l.rfind(self.search_string.as_str()) {
                                line_num = ln;
                                col = c + 1;
                                break;  // Found it
                            }
                        }
                        _ => {
                            return;
                        }
                    }
                }
            }
        }

        if line_num != 0 {
            info!("Found '{}' in line {}",
                  self.search_string,
                  line_num);
            self.set_cursor(line_num, col);
        } else {
            info!("Did not found: {}", self.search_string);
        }

        self.update();
    }

    fn move_next_word(&mut self) {
        let text_copy = self.text.clone();  // so we can borrow self as mutable
        let mut lines = text_copy.lines().skip(self.cursor.line - 1);
        let mut line_num = self.cursor.line;
        let mut col = 1;

        // Check current line after the cursor
        let (_, rest_line) = lines.next().unwrap().split_at(self.cursor.col);
        match rest_line.find(' ') {
            Some(c) => {
                col = c + self.cursor.col + 2;
            }
            None => {
                // If no word break found in current line, go to next
                line_num += 1;
            }
        }

        if line_num <= self.model.get_line_count() {
            info!("Moving to next word at {}:{}", line_num, col);
            self.set_cursor(line_num, col);
            self.update();
        }
    }

    fn move_prev_word(&mut self) {
        let text_copy = self.text.clone();  // so we can borrow self as mutable
        let mut lines = text_copy.lines()
            .rev()
            .skip(self.model.get_line_count() - self.cursor.line);
        let line_num = self.cursor.line;
        let col: usize;

        let line = lines.next();
        if self.cursor.col > 1 {
            // Check current line before the cursor
            let (beg_line, _) = line.unwrap().split_at(self.cursor.col - 2);
            match beg_line.rfind(' ') {
                Some(c) => {
                    col = c + 2;
                }
                None => {
                    // If no word break before cursor in current line, go to
                    // the beginning of the line
                    col = 1;
                }
            }
        } else {
            // If at beginning of line, go to end of previous line
            match lines.next() {
                Some(line) => {
                    self.cursor.col = line.len();
                    self.cursor.line = line_num - 1;
                    self.move_prev_word();

                    // return to avoid set_cursor() below with old line_num
                    return;
                }
                _ => {
                    // Already at beginning of file, nothing to do
                    return;
                }
            }
        }

        info!("Moving to previous word at {}:{}", line_num, col);
        self.set_cursor(line_num, col);
        self.update();
    }


    fn set_current_line(&mut self, line_num: usize) {
        self.cursor.line = line_num;

        let line = match self.text.lines().nth(self.cursor.line - 1) {
            Some(line) => line,
            None => return,
        };
        self.cur_line_len = if self.mode == Mode::Edit {
            line.len() + 1
        } else {
            line.len()
        };

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

    fn set_cursor(&mut self, mut line_num: usize, col: usize) {
        self.focus_col = col;

        self.set_current_line(line_num);
        let line_count = self.model.get_line_count();

        // Update display if line_num is outside of it
        if line_num < self.disp_line ||
           line_num >= self.disp_line + self.height {
            if line_num > line_count - self.height {
                line_num = line_count - self.height + 1;
            }
            let _ = self.display_chunk(line_num, 1);
        }
    }

    fn add_char(&mut self, c: char) {
        let line = self.cursor.line;
        let column = self.cursor.col;
        info!("Add {} at {}:{}", c, line, column);

        self.model.add_char(c, line, column);
        self.text = self.model.get_text();

        let mut disp_line = self.disp_line;
        let disp_col = self.disp_col;

        if c == '\n' {
            // If adding an Enter, we move the cursor to the newline which
            // might fall outside of the display
            let line_num = self.cursor.line + 1;
            self.focus_col = 1;
            self.set_current_line(line_num);
            if self.cursor.line >= disp_line + self.height {
                disp_line += 1;
            }
        } else {
            // If adding any other character move the cursor one past new char
            self.cursor.col += 1;
            self.cur_line_len += 1;
        }
        self.focus_col = self.cursor.col;

        let _ = self.display_chunk(disp_line, disp_col);
        self.update();
    }

    fn delete_char(&mut self) {
        // TODO: Use better data structure for strings. For example, a Rope
        let line = self.cursor.line;
        let column = self.cursor.col;
        info!("Delete char from {}:{}", line, column);

        // Can't delete from the beginning of the file
        if line == 1 && column == 1 {
            return;
        }

        let end_len = self.model.delete_char(line, column);
        self.text = self.model.get_text();

        let mut disp_line = self.disp_line;
        let disp_col = self.disp_col;

        if column == 1 {
            // Removed first character of line, move to line above
            let line_num = self.cursor.line - 1;
            self.set_current_line(line_num);
            self.cursor.col = self.cur_line_len - end_len;
            if self.cursor.line < disp_line {
                disp_line -= 1;
            }
        } else {
            self.cursor.col -= 1;
            self.cur_line_len -= 1;
        }
        self.focus_col = self.cursor.col;

        let _ = self.display_chunk(disp_line, disp_col);
        self.update();
    }

    fn update(&mut self) {
        // Add an informational status line

        let status: String;
        match self.mode {
            Mode::Read | Mode::Edit => {
                let saved = if self.model.get_saved_stat() {
                    '*'
                } else {
                    '.'
                };

                let mode = match self.mode {
                    Mode::Read => 'R',
                    Mode::Edit => 'E',
                    _ => ' ',
                };
                status = format!("{}{} -- {} ({},{})",
                                 mode,
                                 saved,
                                 self.filename,
                                 self.cursor.line,
                                 self.cursor.col);
            }
            Mode::GoToLine => {
                if self.line_jump == 0 {
                    status = String::from(":");
                } else {
                    status = format!(":{}", self.line_jump);
                }
            }
            Mode::Search => {
                if self.search_string.is_empty() {
                    status = String::from("/");
                } else {
                    status = format!("/{}", self.search_string);
                }
            }
        }

        let cur_col = if self.cursor.col == 0 {
            0
        } else {
            (self.cursor.col - self.disp_col) as isize
        };
        self.rustbox.set_cursor(cur_col,
                                (self.cursor.line - self.disp_line) as isize);

        let help: &'static str = "Press 'q' to quit";

        let mut empty = String::with_capacity(self.width - status.len() -
                                              help.len());
        for _ in 0..empty.capacity() {
            empty.push(' ');
        }

        self.rustbox.print(RB_COL_START,
                           self.height,
                           rustbox::RB_REVERSE,
                           Color::White,
                           Color::Black,
                           status.as_ref());
        self.rustbox.print(RB_COL_START + status.len(),
                           self.height,
                           rustbox::RB_NORMAL,
                           Color::White,
                           Color::Black,
                           empty.as_ref());
        self.rustbox.print(self.width - help.len(),
                           self.height,
                           rustbox::RB_REVERSE,
                           Color::White,
                           Color::Black,
                           help);

        self.rustbox.present();
    }
}
