// Viewer

extern crate rustbox;
extern crate time;
extern crate slog_stream;

#[allow(unused_imports)]
use colorconfig;
#[allow(unused_imports)]
use keyconfig;

use model;

use rustbox::{Color, RustBox, OutputMode};
use rustbox::Key;
use std::collections::HashMap;
use std::default::Default;

mod errors {}

use errors::*;

const RB_COL_START: usize = 0;
const RB_ROW_START: usize = 0;
const DEFAULT_TAB_SPACES: usize = 4;

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
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
    MoveStartFile,
    MoveEndFile,
    GoToLine,
    Search,
    SearchNext,
    SearchPrevious,
    MoveNextWord,
    MovePrevWord,
    KillLine,
    KillEndLine,
    Delete,
    CopyStartMark,
    CopyEndMark,
    Paste,
    EditMode,
    ReadMode,
    Append,
    Undo,
    ToggleLineNumbers,
    Save,
    Quit,
    Help,
}

#[derive(Eq,PartialEq)]
enum Mode {
    Read,
    Edit,
    GoToLine,
    Search,
    Help,
}

pub struct Cursor {
    line: usize,
    col: usize,
}

pub struct Colors {
    pub fg: Color,
    pub bg: Color,
    pub line_num: Color,
    pub error: Color,
}

pub struct Viewer {
    rustbox: RustBox,
    text: String,
    current_line: String,
    model: model::Model,
    mode: Mode,
    actions: HashMap<Key, Action>,
    height: usize, // window height without status line
    width: usize,
    show_line_num: bool,
    insert_tab_char: bool,
    tab_size: usize,
    filename: String,
    disp_line: usize, // first displayed line
    disp_col: usize, // first displayed col
    cur_line_len: usize,
    num_lines_digits: usize,
    line_jump: usize,
    cursor: Cursor,
    text_col: usize,
    colors: Colors,
    search_string: String,
    copy_start: Cursor,
    copy_string: String,
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
    pub fn new(filepath: &str,
               filename: String,
               key_map: HashMap<Key, Action>,
               colors: Colors,
               show_line_num: bool,
               insert_tab_char: bool,
               tab_size: usize)
               -> Viewer {
        let mut rustbox = RustBox::init(Default::default()).unwrap();
        let height = rustbox.height() - 1;
        rustbox.set_output_mode(OutputMode::EightBit);
        info!("Terminal window height: {}", height);

        rustbox.set_cursor(RB_COL_START as isize, RB_ROW_START as isize);

        let cursor = Cursor { line: 1, col: 1 };
        let copy_start = Cursor { line: 1, col: 1 };
        let model = model::Model::new(filepath);

        let num_lines_digits = number_of_digits(model.get_line_count());

        let width = if show_line_num {
            rustbox.width() - num_lines_digits - 1
        } else {
            rustbox.width()
        };

        let checked_ts = if tab_size == 0 {
            DEFAULT_TAB_SPACES
        } else {
            tab_size
        };
        info!("The tab size is {}", checked_ts);

        let mut view = Viewer {
            rustbox: rustbox,
            text: String::new(),
            current_line: String::new(),
            model: model,
            mode: Mode::Read,
            actions: key_map,
            height: height,
            width: width,
            show_line_num: show_line_num,
            insert_tab_char: insert_tab_char,
            tab_size: checked_ts,
            filename: filename,
            disp_line: 1,
            disp_col: 1,
            cur_line_len: 1,
            num_lines_digits: num_lines_digits,
            line_jump: 0,
            cursor: cursor,
            text_col: 1,
            colors: colors,
            search_string: String::new(),
            copy_start: copy_start,
            copy_string: String::new(),
        };

        // Check if running for real or a test
        if filepath != "" {
            view.set_current_line(1);
            view.display_chunk(1, 1, true);
            view.update();
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
                     start_col: usize,
                     update_text: bool) {
        if start_line > self.model.get_line_count() {
            warn!("Line {} past EOF", start_line);
        }

        self.disp_line = start_line;
        self.disp_col = start_col;

        self.rustbox.clear();
        if update_text {
            self.text = self.model.get_text_slice(self.disp_line, self.height);
        }

        let mut lines = self.text.lines();
        for ln in 0..(self.height) {
            if let Some(line) = lines.next() {
                self.draw_line(&String::from(line), ln, start_col);

                if self.show_line_num && self.mode != Mode::Help {
                    self.rustbox.print(RB_COL_START + self.width + 1,
                                       ln,
                                       rustbox::RB_NORMAL,
                                       self.colors.line_num,
                                       self.colors.bg,
                                       format!("{}", ln + start_line).as_ref());
                }
            } else {
                info!("Displayed range {} : {} lines",
                      start_line,
                      start_line + ln - 1);
                return;
            }
        }

        info!("Displayed range {} : {} lines",
              start_line,
              start_line + self.height);
    }

    fn draw_line(&self, line: &String, line_num: usize, start_col: usize) {
        let end = start_col + self.width;

        // Check if there is line content to show or past the end
        if line.len() >= start_col {
            let mut print_line = String::new();
            let mut rune_count = 1;
            for c in line.chars() {
                if rune_count >= start_col {
                    if c == '\t' {
                        for _ in 0..self.tab_size {
                            print_line.push(' ');
                        }

                        rune_count += 3;
                    } else {
                        print_line.push(c);
                    }
                }

                rune_count += 1;
                if rune_count > end {
                    break;
                }
            }

            self.rustbox.print(RB_COL_START,
                               line_num,
                               rustbox::RB_NORMAL,
                               self.colors.fg,
                               self.colors.bg,
                               &print_line);
        } else {
            self.rustbox.print(RB_COL_START,
                               line_num,
                               rustbox::RB_NORMAL,
                               self.colors.fg,
                               self.colors.bg,
                               "");
        }
    }

    fn clear_line(&self, line_num: usize) {
        let mut empty = String::new();
        for _ in 0..self.width {
            empty.push(' ');
        }

        self.rustbox.print(RB_COL_START,
                           line_num,
                           rustbox::RB_NORMAL,
                           self.colors.fg,
                           self.colors.bg,
                           &empty);
    }

    fn do_vertical_scroll(&mut self, action: Action) {
        let mut disp_line = self.disp_line;
        let disp_col = self.disp_col;
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
            _ => {
                return;
            }
        }
        self.display_chunk(disp_line, disp_col, true);
    }

    fn consider_horizontal_scroll(&mut self, action: Action) {
        match action {
            Action::MoveDown | Action::MoveUp | Action::MovePageDown |
            Action::MovePageUp | Action::MoveLeft | Action::MoveRight |
            Action::MoveStartLine | Action::MoveEndLine => {
                let tmp_cur_col = if self.cursor.col == 0 {
                    1
                } else {
                    self.cursor.col
                };
                if tmp_cur_col < self.disp_col {
                    // Cursor before display, scroll left
                    let disp_col = tmp_cur_col;
                    let disp_line = self.disp_line;
                    self.display_chunk(disp_line, disp_col, true);
                } else if self.cursor.col > self.disp_col + self.width - 1 {
                    // Cursor past display, scroll right
                    let disp_col = self.cursor.col - self.width + 1;
                    let disp_line = self.disp_line;
                    self.display_chunk(disp_line, disp_col, true);
                }

            }
            _ => {}
        }

    }

    fn move_cursor(&mut self, action: Action) {
        match action {
            Action::MoveDown => {
                self.move_cursor_down(action);
            }
            Action::MoveUp => {
                self.move_cursor_up(action);
            }
            Action::MoveLeft => {
                self.move_cursor_left();
            }
            Action::MoveRight => {
                self.move_cursor_right();
            }
            Action::MovePageDown => {
                self.move_cursor_page_down(action);
            }
            Action::MovePageUp => {
                self.move_cursor_page_up(action);
            }
            Action::MoveStartLine => {
                self.move_cursor_start_line();
            }
            Action::MoveEndLine => {
                self.move_cursor_end_line();
            }
            Action::MoveStartFile => {
                self.move_cursor_start_file();
            }
            Action::MoveEndFile => {
                self.move_cursor_end_file();
            }
            _ => {}
        }

        // Depending on differences in lines we might have to scroll
        // horizontally
        self.consider_horizontal_scroll(action);

        self.update();
    }

    fn move_cursor_down(&mut self, action: Action) {
        let line_count = self.model.get_line_count();

        if self.cursor.line < line_count {
            let tmp = self.cursor.line + 1;
            self.set_current_line(tmp);
            info!("Current line is {}", self.cursor.line);

            if self.cursor.line + 1 > (self.disp_line + self.height) {
                self.do_vertical_scroll(action);
            }
        } else {
            info!("Can't go down, already at the bottom of file");
            return;
        }
    }

    fn move_cursor_up(&mut self, action: Action) {
        if self.cursor.line > 1 {
            let tmp = self.cursor.line - 1;
            self.set_current_line(tmp);
            info!("Current line is {}", self.cursor.line);

            if self.cursor.line < self.disp_line {
                self.do_vertical_scroll(action);
            }
        } else {
            info!("Can't go up, already at the top of file");
            return;
        }
    }

    fn move_cursor_left(&mut self) {
        if self.text_col > 1 {
            self.text_col -= 1;
            self.cursor.col = self.match_cursor_text(self.text_col);
        } else {
            info!("Can't go left, already at beginning of the line");
            return;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.text_col < self.cur_line_len {
            self.text_col += 1;
            self.cursor.col = self.match_cursor_text(self.text_col);
        } else {
            info!("Can't go right, already at end of the line");
            return;
        }
    }

    fn move_cursor_page_down(&mut self, action: Action) {
        let line_count = self.model.get_line_count();

        if self.cursor.line + self.height < line_count {
            let tmp = self.cursor.line + self.height;
            self.set_current_line(tmp);
        } else {
            self.set_current_line(line_count);
        }

        self.do_vertical_scroll(action);
    }

    fn move_cursor_page_up(&mut self, action: Action) {
        if self.cursor.line > self.height {
            let tmp = self.cursor.line - self.height;
            self.set_current_line(tmp);
        } else {
            self.set_current_line(1);
        }

        self.do_vertical_scroll(action);
    }

    fn move_cursor_start_line(&mut self) {
        if self.cur_line_len > 0 {
            self.cursor.col = 1;
            self.text_col = 1;
        } else {
            info!("Can't move to the beginning of an empty line");
        }
    }

    fn move_cursor_end_line(&mut self) {
        if self.cur_line_len > 0 {
            self.text_col = self.cur_line_len;
            self.cursor.col = self.match_cursor_text(self.text_col);
        } else {
            info!("Can't move to the end of an empty line");
        }
    }

    fn move_cursor_start_file(&mut self) {
        self.cursor.col = 1;
        self.text_col = 1;
        self.set_current_line(1);

        if self.disp_line > 1 || self.disp_col > 1 {
            self.display_chunk(1, 1, true);
        }
    }

    fn move_cursor_end_file(&mut self) {
        let height = self.height;
        let line_count = self.model.get_line_count();
        self.set_current_line(line_count);

        self.text_col = self.cur_line_len;
        self.cursor.col = self.match_cursor_text(self.text_col);

        if self.disp_line + height <= line_count {
            self.display_chunk(line_count - height + 1, 1, true);
        }
        self.consider_horizontal_scroll(Action::MovePageDown);
    }

    fn match_key_action(&mut self, key: Key) -> bool {
        let no_action = Action::None;
        let action = *self.actions.get(&key).unwrap_or(&no_action);

        match self.mode {
            Mode::Edit => {
                return self.match_key_action_edit(action, key);
            }
            Mode::Read => {
                return self.match_key_action_read(action);
            }
            Mode::GoToLine => {
                match action {
                    Action::Go => {
                        self.do_line_jump();
                    }
                    _ => {
                        // Numbers don't always match GoToLine action
                        self.go_to_line_mode(&key);
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
            Mode::Help => {
                if action == Action::Quit {
                    self.exit_help();
                }
            }
        }

        true
    }

    #[allow(dead_code)]
    fn get_cursor(&self) -> Cursor {
        Cursor {
            line: self.cursor.line,
            col: self.cursor.col,
        }
    }

    fn match_key_action_edit(&mut self, action: Action, key: Key) -> bool {
        match action {
            Action::Quit => {
                if self.confirm_quit() {
                    return false;
                }
            }
            Action::MoveUp | Action::MoveDown | Action::MoveLeft |
            Action::MoveRight | Action::MovePageDown | Action::MovePageUp |
            Action::MoveStartLine | Action::MoveEndLine => {
                self.move_cursor(action);
            }
            Action::ReadMode => {
                self.switch_mode(action);
            }
            Action::Save => {
                self.model.save();
                self.update();
            }
            Action::KillLine => {
                self.delete_line();
            }
            Action::KillEndLine => {
                self.delete_end_of_line();
            }
            Action::CopyStartMark => {
                self.select_copy_start_marker();
            }
            Action::CopyEndMark => {
                self.select_copy_end_marker();
            }
            Action::Paste => {
                self.paste();
            }
            Action::Undo => {
                self.undo();
            }
            Action::ToggleLineNumbers => {
                self.toggle_line_numbers();
            }
            Action::Help => {
                self.show_help();
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
                        self.delete_backspace();
                    }
                    Key::Delete => {
                        self.delete_at_cursor();
                    }
                    Key::Tab => {
                        if self.insert_tab_char {
                            self.add_char('\t');
                        } else {
                            self.add_tab_spaces();
                        }
                    }
                    Key::Ctrl('t') => {
                        self.add_char('\t');
                    }
                    _ => {}
                }
            }
        }

        true
    }

    fn match_key_action_read(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => {
                if self.confirm_quit() {
                    return false;
                }
            }
            Action::MoveUp | Action::MoveDown | Action::MoveLeft |
            Action::MoveRight | Action::MovePageDown | Action::MovePageUp |
            Action::MoveStartLine | Action::MoveEndLine |
            Action::MoveStartFile | Action::MoveEndFile => {
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
            Action::KillLine => {
                self.delete_line();
            }
            Action::KillEndLine => {
                self.delete_end_of_line();
            }
            Action::Delete => {
                self.delete_at_cursor();
            }
            Action::CopyStartMark => {
                self.select_copy_start_marker();
            }
            Action::CopyEndMark => {
                self.select_copy_end_marker();
            }
            Action::EditMode => {
                self.switch_mode(action);
            }
            Action::Append => {
                self.match_key_action_read(Action::EditMode);
                self.move_cursor(Action::MoveRight);
            }
            Action::ToggleLineNumbers => {
                self.toggle_line_numbers();
            }
            Action::Save => {
                self.model.save();
                self.update();
            }
            Action::Help => {
                self.show_help();
            }
            _ => {}
        }

        true
    }

    fn switch_mode(&mut self, action: Action) {
        let cur_line = self.cursor.line;

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

        // Update current line max column
        self.set_current_line(cur_line);

        self.update();
    }

    fn go_to_line_mode(&mut self, key: &Key) {
        let n = match *key {
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
            Key::Backspace => {
                self.line_jump /= 10;
                self.update();
                return;
            }
            _ => {
                return;
            }
        };

        self.line_jump = (self.line_jump * 10) + n;
        self.update();
    }

    fn do_line_jump(&mut self) {
        let line_num = self.line_jump;

        self.mode = Mode::Read; // Set back to previous mode
        self.line_jump = 0;

        if line_num > self.model.get_line_count() || line_num == 0 {
            info!("ERROR: Invalid line number {}", line_num);
            self.update();

            return;
        }

        let col = self.match_cursor_text(1);

        info!("Go to line {}", line_num);
        self.set_cursor(line_num, col);
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

        let cur_pos = model::Position {
            line: self.cursor.line,
            col: self.text_col,
        };
        let pos = self.model
            .forward_search(self.search_string.as_str(), cur_pos);

        if pos.line != 0 {
            self.text_col = pos.col;
            info!("Found '{}' in line {} column {}",
                  self.search_string,
                  pos.line,
                  pos.col);
            self.set_cursor(pos.line, pos.col);
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
        let cur_pos = model::Position {
            line: self.cursor.line,
            col: self.text_col,
        };
        let pos = self.model
            .backward_search(self.search_string.as_str(), cur_pos);

        if pos.line != 0 {
            self.text_col = pos.col;
            info!("Found '{}' in line {} column {}",
                  self.search_string,
                  pos.line,
                  pos.col);
            self.set_cursor(pos.line, pos.col);
        } else {
            info!("Did not found: {}", self.search_string);
        }

        self.update();
    }

    fn move_next_word(&mut self) {
        let mut line_num = self.cursor.line;
        let mut col = 1;
        {
            let mut lines = self.text.lines().skip(self.cursor.line - 1);
            let word_break: &[_] = &[' ', '\t'];

            // Check current line after the cursor
            // TODO: Don't consider a tab as a word.
            //       Check next character is alphanumeric
            let (_, rest_line) = lines.next().unwrap().split_at(self.text_col);
            match rest_line.find(word_break) {
                Some(c) => {
                    col = c + self.text_col + 2;
                }
                None => {
                    // If no word break found in current line, go to next
                    line_num += 1;
                }
            }
        }

        if line_num <= self.model.get_line_count() {
            info!("Moving to next word at {}:{}", line_num, col);
            self.set_cursor(line_num, col);
            self.update();
        }
    }

    fn move_prev_word(&mut self) {
        let text_copy = self.text.clone(); // so we can borrow self as mutable
        let mut lines = text_copy.lines()
            .rev()
            .skip(self.model.get_line_count() - self.cursor.line);
        let line_num = self.cursor.line;
        let col: usize;
        // TODO: Check next character is alphanumeric
        let word_break: &[_] = &[' ', '\t'];

        let line = lines.next();
        if self.text_col > 1 {
            // Check current line before the cursor
            let (beg_line, _) = line.unwrap().split_at(self.text_col - 2);
            match beg_line.rfind(word_break) {
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
                    self.text_col = line.len();
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

        self.current_line = self.model.get_line(self.cursor.line);;
        self.cur_line_len = if self.mode == Mode::Edit {
            self.current_line.len() + 1
        } else {
            self.current_line.len()
        };

        self.cursor.col = self.match_cursor_text(self.text_col);

        if self.cur_line_len < self.text_col {
            // previous line was longer
            self.cursor.col = self.match_cursor_text(self.cur_line_len);
            self.text_col = self.cur_line_len;
        } else if self.cursor.col == 0 {
            // previous line was empty
            self.cursor.col = 1; // jump back to first column
            self.text_col = 1;
        }
    }

    // Match the cursor column based on current text line position
    fn match_cursor_text(&self, text_col: usize) -> usize {
        let ref line = self.current_line;
        let mut count = 0;
        let mut chars = line.chars();
        for n in 0..text_col {
            if let Some(c) = chars.next() {
                if c == '\t' {
                    count += self.tab_size;
                } else {
                    count += 1;
                }
            } else if n == line.len() {
                // Extra char for editing mode
                count += 1;
            }
        }

        count
    }

    // Match the text column based on the current cursor position
    #[allow(dead_code)]
    fn match_text_cursor(&self, cursor_col: usize) -> usize {
        let mut count = 0;
        let mut chars = self.current_line.chars();
        let mut n = 0;

        while n < cursor_col {
            if let Some(c) = chars.next() {
                if c == '\t' {
                    n += self.tab_size;
                } else {
                    n += 1;
                }

                count += 1;
            }
        }

        count
    }

    fn set_cursor(&mut self, mut line_num: usize, col: usize) {
        self.text_col = col;

        self.set_current_line(line_num);
        let line_count = self.model.get_line_count();

        // Update display if line_num is outside of it
        if line_num < self.disp_line ||
           line_num >= self.disp_line + self.height {
            if line_num > line_count - self.height {
                line_num = line_count - self.height + 1;
            }
            self.display_chunk(line_num, 1, true);
        }
    }

    fn update_num_lines_digits(&mut self, add: bool, single: bool) -> bool {
        let line_count = self.model.get_line_count();
        let num_lines_digits: usize;

        if single {
            if (add && (line_count % 10) == 0) ||
               (!add && (line_count % 10) == 9) {
                num_lines_digits = number_of_digits(line_count);
            } else {
                return false;
            }
        } else {
            num_lines_digits = number_of_digits(line_count);
        }

        if self.num_lines_digits != num_lines_digits {
            self.num_lines_digits = num_lines_digits;
            return true;
        }

        false
    }

    fn add_char(&mut self, c: char) {
        info!("Add '{}' at {}:{}",
              c,
              self.cursor.line,
              self.text_col);

        self.model.add_char(c, self.cursor.line, self.text_col);
        self.update_after_add(c, 1);
    }

    fn add_tab_spaces(&mut self) {
        let tab_size = self.tab_size;
        info!("Add tab spaces at {}:{}", self.cursor.line, self.text_col);

        for c in 0..tab_size {
            self.model.add_char(' ', self.cursor.line, self.text_col + c);
        }
        self.update_after_add(' ', tab_size);
    }

    fn update_after_add(&mut self, c: char, count: usize) {
        self.update_text(false);

        let mut disp_line = self.disp_line;
        let mut disp_col = self.disp_col;
        let mut line_num = self.cursor.line;
        let mut disp_col_change = false;

        if c == '\n' {
            // If adding an Enter, we move the cursor to the newline which
            // might fall outside of the display
            line_num += 1;
            disp_col = 1;
            self.text_col = 1;
            self.set_current_line(line_num);
            if self.cursor.line >= disp_line + self.height {
                disp_line += 1;
            }

            if self.show_line_num && self.update_num_lines_digits(true, true) {
                self.width = self.rustbox.width() - self.num_lines_digits - 1;
            }
        } else {
            // If adding any other character move the cursor one past new char
            self.text_col += count;
            self.cur_line_len += count;
        }
        self.current_line = self.model.get_line(self.cursor.line);
        self.cursor.col = self.match_cursor_text(self.text_col);

        if self.cursor.col > disp_col + self.width {
            disp_col = self.cursor.col - self.width;
            disp_col_change = true;
        }

        if c == '\n' || disp_col_change {
            self.display_chunk(disp_line, disp_col, true);
        } else {
            line_num = line_num - disp_line;
            self.draw_line(&self.current_line, line_num, disp_col);
        }

        self.update();
    }

    fn delete_char(&mut self, backspace: bool) {
        // TODO: Use better data structure for strings. For example, a Rope
        let mut line_num = self.cursor.line;
        let column = if backspace {
            self.text_col
        } else {
            self.text_col + 1
        };

        // Can't delete char from the beginning of the file or past the line
        // and can't do Delete action past the line (Edit Mode)
        if backspace {
            if (line_num == 1 && column == 1) ||
               self.text_col - 1 > self.cur_line_len {
                return;
            }
        } else if self.cur_line_len == 0 ||
                  (self.mode == Mode::Edit &&
                   (self.text_col > self.cur_line_len)) {
            return;
        }

        info!("Delete char from {}:{}", line_num, column);
        let end_len = self.model.delete_char(line_num, column);
        self.update_text(false);

        let mut disp_line = self.disp_line;
        let mut disp_col = self.disp_col;

        if backspace {
            if column == 1 {
                // Removed first character of line, move to line above
                line_num -= 1;
                self.set_current_line(line_num);
                self.text_col = self.cur_line_len - end_len;
                self.cursor.col = self.match_cursor_text(self.text_col);
                if self.cursor.line < disp_line {
                    disp_line -= 1;
                }

                if self.show_line_num &&
                   self.update_num_lines_digits(false, true) {
                    self.width = self.rustbox.width() - self.num_lines_digits -
                                 1;
                }
            } else {
                self.text_col -= 1;
            }
        } else if column == self.cur_line_len + 1 {
            // Move cursor back if doing Delete in last character in line
            self.text_col -= 1;
        }

        self.current_line = self.model.get_line(self.cursor.line);
        self.cur_line_len = if self.mode == Mode::Edit {
            self.current_line.len() + 1
        } else {
            self.current_line.len()
        };

        self.cursor.col = self.match_cursor_text(self.text_col);
        if self.cursor.col < disp_col {
            disp_col = self.cursor.col;
            self.display_chunk(disp_line, disp_col, true);
        }

        if backspace && column == 1 {
            self.display_chunk(disp_line, disp_col, true);
        } else {
            line_num -= disp_line;
            self.clear_line(line_num);
            self.draw_line(&self.current_line, line_num, disp_col);
        }
        self.update();
    }

    fn delete_backspace(&mut self) {
        if self.text_col > self.tab_size {
            // Check if we should delete an indentation level
            let mut line_num = self.cursor.line;
            let line = self.model.get_line(line_num);
            let (beg_line, _) = line.split_at(self.text_col - 1);

            let mut tab_space = String::new();
            for _ in 0..self.tab_size {
                tab_space.push(' ');
            }
            let len = beg_line.len();
            if beg_line[len - self.tab_size..len] == tab_space {
                self.model
                    .delete_block(self.cursor.line,
                                  self.text_col,
                                  self.tab_size);

                self.cursor.col -= self.tab_size;
                self.text_col -= self.tab_size;

                let disp_line = self.disp_line;
                let disp_col = self.disp_col;
                self.update_text(false);
                self.set_current_line(line_num);

                line_num -= disp_line;
                self.clear_line(line_num);
                self.draw_line(&self.current_line, line_num, disp_col);

                self.update();

                return;
            }
        }
        self.delete_char(true);
    }

    fn delete_at_cursor(&mut self) {
        self.delete_char(false);
    }

    fn select_copy_start_marker(&mut self) {
        info!("Select copy start marker {}:{}",
              self.cursor.line,
              self.cursor.col);

        self.copy_start.line = self.cursor.line;
        self.copy_start.col = self.text_col;
    }

    fn select_copy_end_marker(&mut self) {
        info!("Select copy end marker {}:{}",
              self.cursor.line,
              self.text_col);

        if self.cursor.line < self.copy_start.line ||
           (self.cursor.line == self.copy_start.line &&
            self.text_col < self.copy_start.line) {
            error!("Copy end marker can't be before start marker");
        }

        let text_copy = self.text.clone();
        let mut lines = text_copy.lines().skip(self.copy_start.line - 1);

        if self.copy_start.line == self.cursor.line {
            let line = lines.next().unwrap();
            // Check cursor isn't past the line
            let col = if self.cursor.col <= line.len() {
                self.text_col
            } else {
                line.len()
            };

            self.copy_string = line[self.copy_start.col - 1..col].to_string();
        } else {
            self.copy_string = String::new();

            let (_, start) =
                lines.next().unwrap().split_at(self.copy_start.col - 1);
            self.copy_string.push_str(start);
            self.copy_string.push('\n');

            let mid_lines = self.cursor.line - self.copy_start.line - 1;
            if mid_lines > 0 {
                for _ in 0..mid_lines {
                    self.copy_string.push_str(lines.next().unwrap());
                    self.copy_string.push('\n');
                }
            }

            // Check cursor isn't past the last line
            let last_line = lines.next().unwrap();
            let col = if self.text_col <= last_line.len() {
                self.text_col
            } else {
                last_line.len()
            };
            let (end, _) = last_line.split_at(col);
            self.copy_string.push_str(end);
        }
    }

    fn paste(&mut self) {
        info!("Paste copy string");
        let line = self.cursor.line;
        let column = self.cursor.col;
        let disp_line = self.disp_line;
        let disp_col = self.disp_col;
        let paste_lines = self.copy_string.lines().count();

        let copy_string = self.copy_string.clone();

        self.model.add_block(copy_string, line, column);
        self.update_text(false);

        if self.show_line_num && self.update_num_lines_digits(true, false) {
            self.width = self.rustbox.width() - self.num_lines_digits - 1;
        }

        self.set_current_line(line + paste_lines - 1);
        self.display_chunk(disp_line, disp_col, true);
        self.update();
    }

    fn undo(&mut self) {
        info!("Undo last change");
        let mut disp_line = self.disp_line;
        let disp_col = self.disp_col;

        self.model.undo();
        self.update_text(false);

        if self.show_line_num && self.update_num_lines_digits(true, false) {
            self.width = self.rustbox.width() - self.num_lines_digits - 1;
        }

        let line_count = self.model.get_line_count();
        if self.cursor.line > line_count {
            self.move_cursor_end_file();
        }

        if self.cursor.line < disp_line {
            if line_count > self.height {
                disp_line = line_count - self.height + 1;
            } else {
                disp_line = 1;
            }
        }
        self.display_chunk(disp_line, disp_col, true);
        self.update();
    }

    fn delete_line(&mut self) {
        let line = self.cursor.line;
        let line_count = self.model.get_line_count();

        let line_num = if self.cursor.line != line_count {
            self.cursor.line
        } else {
            self.cursor.line - 1
        };

        if !self.model.delete_line(line) {
            return;
        }
        self.update_text(false);

        if self.show_line_num && self.update_num_lines_digits(false, true) {
            self.width = self.rustbox.width() - self.num_lines_digits - 1;
        }

        self.set_current_line(line_num);

        self.text_col = 1;
        self.cursor.col = 1;
        let disp_line = self.disp_line;
        self.display_chunk(disp_line, 1, true);
        self.update();
    }

    fn delete_end_of_line(&mut self) {
        let line = self.cursor.line;
        let line_len = self.cur_line_len;
        let col = self.text_col;

        if col == line_len {
            return;
        }

        // Account for having one extra character in Edit Mode
        if self.mode == Mode::Edit {
            self.model.delete_block(line, line_len, line_len - col - 1);
            self.cur_line_len -= line_len - col - 1;
        } else {
            self.model.delete_block(line, line_len + 1, line_len - col);
            self.cur_line_len -= line_len - col;
        }

        self.update_text(false);

        let disp_line = self.disp_line;
        let disp_col = self.disp_col;
        self.display_chunk(disp_line, disp_col, true);
        self.update();
    }

    fn toggle_line_numbers(&mut self) {
        let disp_line = self.disp_line;
        let disp_col = self.disp_col;

        self.show_line_num = !self.show_line_num;
        info!("Toggle showing line numbers: {}",
              self.show_line_num);

        self.width = if self.show_line_num {
            self.rustbox.width() - self.num_lines_digits - 1
        } else {
            self.rustbox.width()
        };

        self.display_chunk(disp_line, disp_col, true);
        self.update();
    }

    fn confirm_quit(&mut self) -> bool {
        if self.model.get_saved_stat() {
            info!("Quitting application");
            return true;
        }

        let prompt: &'static str = "Unsaved changes. Quit? (y/s/n)";
        let mut empty = String::with_capacity(self.rustbox.width() -
                                              prompt.len());
        for _ in 0..empty.capacity() {
            empty.push(' ');
        }
        self.rustbox.print(RB_COL_START,
                           self.height,
                           rustbox::RB_NORMAL,
                           self.colors.fg,
                           self.colors.bg,
                           empty.as_ref());
        self.rustbox.print(self.rustbox.width() - prompt.len(),
                           self.height,
                           rustbox::RB_REVERSE,
                           self.colors.fg,
                           self.colors.bg,
                           prompt);
        self.rustbox.present();

        match self.rustbox.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Key::Char('y') | Key::Char('Y') => {
                        return true;
                    }
                    Key::Char('s') | Key::Char('S') => {
                        self.model.save();
                        return true;
                    }
                    _ => {}
                }
            }
            Err(_) => {
                error!("Something went wrong polling for Rustbox events");
            }
            _ => {}
        }

        self.update();
        false
    }

    fn update_text(&mut self, update_line_len: bool) {
        self.text = self.model.get_text_slice(self.disp_line, self.height);

        if update_line_len {
            self.current_line = self.model
                .get_line(self.cursor.line + self.disp_line - 1);
            self.cur_line_len = if self.mode == Mode::Edit {
                self.current_line.len() + 1
            } else {
                self.current_line.len()
            };
        }
    }

    fn show_help(&mut self) {
        let mut help_text = String::from("  ::  Ente Help  ::

List of available keys:\n");
        for (key, act) in &self.actions {
            help_text.push_str(format!("{:?} => {:?}\n", key, act).as_ref());
        }

        self.text = help_text;
        self.mode = Mode::Help;

        self.cursor.line = 1;
        self.cursor.col = 1;

        self.display_chunk(1, 1, false);
        self.update();
    }

    fn exit_help(&mut self) {
        self.mode = Mode::Read;
        self.text = self.model.get_text_slice(self.disp_line, self.height);

        self.display_chunk(1, 1, true);
        self.update();
    }

    fn update(&mut self) {
        // Add an informational status line

        let status: String;
        match self.mode {
            Mode::Read | Mode::Edit => {
                let saved = if self.model.get_saved_stat() {
                    '.'
                } else {
                    '*'
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
            Mode::Help => {
                status = String::from("Press 'Ctrl-q' to exit help");
            }
        }

        let cur_col = if self.cursor.col == 0 {
            0
        } else {
            (self.cursor.col - self.disp_col) as isize
        };
        self.rustbox.set_cursor(cur_col,
                                (self.cursor.line - self.disp_line) as isize);

        let perc = format!(" {}% ",
                           (self.cursor.line * 100) /
                           self.model.get_line_count());

        let help = String::from("F1 for help");

        let mut first_empty = String::with_capacity((self.rustbox.width() / 2) -
                                                    status.len() -
                                                    2);
        for _ in 0..first_empty.capacity() {
            first_empty.push(' ');
        }
        let mut second_empty = String::with_capacity((self.rustbox.width() /
                                                      2) -
                                                     (perc.len() / 2) -
                                                     help.len());
        for _ in 0..second_empty.capacity() {
            second_empty.push(' ');
        }

        self.rustbox.print(RB_COL_START,
                           self.height,
                           rustbox::RB_REVERSE,
                           self.colors.fg,
                           self.colors.bg,
                           status.as_ref());
        if self.mode == Mode::Help {
            self.rustbox.present();
            return;
        }

        self.rustbox.print(RB_COL_START + status.len(),
                           self.height,
                           rustbox::RB_NORMAL,
                           self.colors.fg,
                           self.colors.bg,
                           first_empty.as_ref());
        self.rustbox.print(RB_COL_START + status.len() + first_empty.len(),
                           self.height,
                           rustbox::RB_REVERSE,
                           self.colors.fg,
                           self.colors.bg,
                           perc.as_ref());
        self.rustbox.print(RB_COL_START + status.len() + first_empty.len() +
                           perc.len(),
                           self.height,
                           rustbox::RB_NORMAL,
                           self.colors.fg,
                           self.colors.bg,
                           second_empty.as_ref());
        self.rustbox.print(RB_COL_START + status.len() + first_empty.len() +
                           perc.len() +
                           second_empty.len(),
                           self.height,
                           rustbox::RB_REVERSE,
                           self.colors.fg,
                           self.colors.bg,
                           help.as_ref());

        self.rustbox.present();
    }
}

impl Colors {
    pub fn new() -> Colors {
        Colors {
            fg: Color::White,
            bg: Color::Black,
            line_num: Color::Blue,
            error: Color::Red,
        }
    }
}

#[test]
fn test_new() {
    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();

    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(String::from("test"));
    test_view.update_text(true);

    // test_view.display_chunk(1, 1, true);
    assert_eq!("test\n", test_view.text);
    assert_eq!(1, test_view.cursor.col);
    assert_eq!(1, test_view.cursor.line);
}

#[allow(dead_code)]
fn compare_cursors(first: &Cursor, second: &Cursor) -> bool {
    first.col == second.col && first.line == second.line
}

#[test]
fn test_cursor() {
    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_cursor = Cursor { line: 1, col: 1 };
    let mut view_cursor: Cursor;

    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(String::from("test
second line"));

    // Init at 1,1
    view_cursor = test_view.get_cursor();
    assert!(compare_cursors(&view_cursor, &test_cursor));

    // Move Down
    test_view.move_cursor(Action::MoveDown);
    view_cursor = test_view.get_cursor();
    test_cursor.line = 2;
    assert!(compare_cursors(&view_cursor, &test_cursor));

    // Move Left, twice
    test_view.move_cursor(Action::MoveLeft);
    test_view.move_cursor(Action::MoveLeft);
    view_cursor = test_view.get_cursor();
    test_cursor.col = 1;
    assert!(compare_cursors(&view_cursor, &test_cursor));

    // Move Right, thrice
    test_view.move_cursor(Action::MoveRight);
    test_view.move_cursor(Action::MoveRight);
    test_view.move_cursor(Action::MoveRight);
    view_cursor = test_view.get_cursor();
    test_cursor.col = 4;
    assert!(compare_cursors(&view_cursor, &test_cursor));

    // Move Up
    test_view.move_cursor(Action::MoveUp);
    view_cursor = test_view.get_cursor();
    test_cursor.line = 1;
    assert!(compare_cursors(&view_cursor, &test_cursor));
}

#[test]
fn test_pageup_pagedown() {
    let mut text = String::from("test");
    // Increase number of lines in text
    for _ in 0..200 {
        text.push_str("line\n");
    }

    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_cursor = Cursor { line: 1, col: 1 };
    let mut view_cursor: Cursor;

    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);

    // Init at 1,1
    view_cursor = test_view.get_cursor();
    assert!(compare_cursors(&view_cursor, &test_cursor));

    // Move Page Down
    test_view.move_cursor(Action::MovePageDown);
    view_cursor = test_view.get_cursor();
    test_cursor.line = 1 + test_view.height;
    assert!(compare_cursors(&view_cursor, &test_cursor));

    // Move Page Up
    test_view.move_cursor(Action::MovePageUp);
    test_view.move_cursor(Action::MovePageUp);
    view_cursor = test_view.get_cursor();
    test_cursor.line = 1;
    assert!(compare_cursors(&view_cursor, &test_cursor));
}

#[test]
fn test_start_end_line() {
    let text = String::from("this text is 30 characters long");
    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_cursor = Cursor { line: 1, col: 1 };
    let mut view_cursor: Cursor;

    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);
    test_view.update_text(true);

    // Init at 1,1
    view_cursor = test_view.get_cursor();
    assert!(compare_cursors(&view_cursor, &test_cursor));

    // Move End Line
    test_view.move_cursor(Action::MoveEndLine);
    view_cursor = test_view.get_cursor();
    test_cursor.col = 31;
    assert!(compare_cursors(&view_cursor, &test_cursor));

    // Move Start Line
    test_view.move_cursor(Action::MoveStartLine);
    view_cursor = test_view.get_cursor();
    test_cursor.col = 1;
    assert!(compare_cursors(&view_cursor, &test_cursor));
}

#[test]
fn test_start_end_file() {
    let mut text = String::from("");
    // Increase number of lines in text
    for n in 0..50 {
        text.push_str(format!("line {}\n", n).as_ref());
    }

    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_cursor = Cursor { line: 1, col: 1 };
    let mut view_cursor: Cursor;

    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);

    // Init at 1,1
    view_cursor = test_view.get_cursor();
    assert!(compare_cursors(&view_cursor, &test_cursor));

    // Move End Line
    test_view.move_cursor(Action::MoveEndFile);
    view_cursor = test_view.get_cursor();
    test_cursor.line = 50;
    test_cursor.col = 7;
    assert!(compare_cursors(&view_cursor, &test_cursor));

    // Move Start Line
    test_view.move_cursor(Action::MoveStartFile);
    view_cursor = test_view.get_cursor();
    test_cursor.col = 1;
    test_cursor.line = 1;
    assert!(compare_cursors(&view_cursor, &test_cursor));
}

#[test]
fn test_match_cursor_text() {
    let text = String::from("abc\tdef\tg");
    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);
    test_view.update_text(true);

    assert_eq!(3, test_view.match_cursor_text(3));
    assert_eq!(4 + test_view.tab_size, test_view.match_cursor_text(5));
    assert_eq!(7 + (test_view.tab_size * 2),
               test_view.match_cursor_text(9));
    assert_eq!(1 + 7 + (test_view.tab_size * 2),
               test_view.match_cursor_text(20));
}

#[test]
fn test_match_cursor_text_start_with_tab() {
    let text = String::from("\t\t\t_test");
    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);
    test_view.update_text(true);

    assert_eq!(test_view.tab_size, test_view.match_cursor_text(1));
    assert_eq!(test_view.tab_size * 2, test_view.match_cursor_text(2));
    assert_eq!(test_view.tab_size * 3, test_view.match_cursor_text(3));
    assert_eq!(1 + (test_view.tab_size * 3),
               test_view.match_cursor_text(4));
    assert_eq!(2 + (test_view.tab_size * 3),
               test_view.match_cursor_text(5));
    assert_eq!(5 + (test_view.tab_size * 3),
               test_view.match_cursor_text(8));
    assert_eq!(6 + (test_view.tab_size * 3),
               test_view.match_cursor_text(9));
}

#[test]
fn test_set_current_line() {
    let text = String::from("First
Second length 16
\tThird Line length 20

Fifth 7");
    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);
    test_view.update_text(true);

    test_view.set_current_line(2); // move to line 2
    assert_eq!(16, test_view.cur_line_len);
    test_view.set_current_line(3); // move to line 3 (with tab)
    test_view.move_cursor(Action::MoveEndLine); // move to end of line
    assert_eq!(21, test_view.cur_line_len);
    assert_eq!(24, test_view.cursor.col);

    test_view.set_current_line(5); // move to shorter line 5
    assert_eq!(7, test_view.cur_line_len);
    assert_eq!(7, test_view.cursor.col);

    test_view.mode = Mode::Edit; // change mode to edit

    test_view.set_current_line(2); // move to line 2
    assert_eq!(17, test_view.cur_line_len);
    assert_eq!(7, test_view.cursor.col);
    test_view.move_cursor(Action::MoveEndLine); // move to end of line
    assert_eq!(17, test_view.cursor.col);
    test_view.set_current_line(3); // move to line 3
    assert_eq!(22, test_view.cur_line_len);
    test_view.move_cursor(Action::MoveEndLine); // move to end of line

    test_view.set_current_line(5); // move to shorter line 5
    assert_eq!(8, test_view.cur_line_len);

    test_view.move_cursor_left(); // move left twice
    test_view.move_cursor_left();
    assert_eq!(6, test_view.cursor.col);

    test_view.set_current_line(4); // move to empty line 4
    assert_eq!(1, test_view.cursor.col);
}

#[test]
fn test_match_text_cursor() {
    let text = String::from("First line\n\tthis is a tab");
    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);
    test_view.update_text(true);

    assert_eq!(1, test_view.match_text_cursor(1));
    assert_eq!(10, test_view.match_text_cursor(10));

    test_view.set_current_line(2);

    assert_eq!(1, test_view.match_text_cursor(4));
    assert_eq!(9, test_view.match_text_cursor(12));
}

#[test]
fn test_do_forward_search() {
    let text = String::from("This is a test
in which we try forward search
third line includes test
\t\tthis test should work with tabs as well
testing last line as well");
    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);
    test_view.update_text(true);

    // first result
    test_view.search_string = String::from("test");
    test_view.do_forward_search();
    assert_eq!(1, test_view.cursor.line);
    assert_eq!(11, test_view.cursor.col);

    // next result
    test_view.do_forward_search();
    assert_eq!(3, test_view.cursor.line);
    assert_eq!(21, test_view.cursor.col);

    // third result
    test_view.do_forward_search();
    assert_eq!(4, test_view.cursor.line);
    assert_eq!(14, test_view.cursor.col);

    // last result
    test_view.do_forward_search();
    assert_eq!(5, test_view.cursor.line);
    assert_eq!(1, test_view.cursor.col);

    // no more results
    test_view.do_forward_search();
    assert_eq!(5, test_view.cursor.line);
    assert_eq!(1, test_view.cursor.col);
}

#[test]
fn test_do_backward_search() {
    let text = String::from("This is a test
in which we try forward search
third line includes test
\t\tthis test should work with tabs as well
testing last line as well");
    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);
    test_view.update_text(true);

    test_view.move_cursor_end_file();
    assert_eq!(5, test_view.cursor.line);
    assert_eq!(25, test_view.cursor.col);

    // first result
    test_view.search_string = String::from("test");
    test_view.do_backward_search();
    assert_eq!(5, test_view.cursor.line);
    assert_eq!(1, test_view.cursor.col);

    // next result
    test_view.do_backward_search();
    assert_eq!(4, test_view.cursor.line);
    assert_eq!(14, test_view.cursor.col);

    // third result
    test_view.do_backward_search();
    assert_eq!(3, test_view.cursor.line);
    assert_eq!(21, test_view.cursor.col);

    // last result
    test_view.do_backward_search();
    assert_eq!(1, test_view.cursor.line);
    assert_eq!(11, test_view.cursor.col);

    // no more results
    test_view.do_backward_search();
    assert_eq!(1, test_view.cursor.line);
    assert_eq!(11, test_view.cursor.col);
}

#[test]
fn test_move_next_word() {
    let text = String::from("This is a test
\t\tfor move next word
third line");
    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);
    test_view.update_text(true);

    // first line
    test_view.move_next_word();
    assert_eq!(6, test_view.cursor.col);
    test_view.move_next_word();
    assert_eq!(9, test_view.cursor.col);
    test_view.move_next_word();
    assert_eq!(11, test_view.cursor.col);

    // second line
    test_view.move_next_word();
    assert_eq!(2, test_view.cursor.line);
    test_view.move_next_word();
    assert_eq!(9, test_view.cursor.col);
    test_view.move_next_word();
    assert_eq!(13, test_view.cursor.col);
    test_view.move_next_word();
    assert_eq!(18, test_view.cursor.col);
    test_view.move_next_word();
    assert_eq!(23, test_view.cursor.col);

    // last line
    test_view.move_next_word();
    assert_eq!(3, test_view.cursor.line);
    assert_eq!(1, test_view.cursor.col);
    test_view.move_next_word();
    assert_eq!(7, test_view.cursor.col);
}

#[test]
fn test_move_prev_word() {
    let text = String::from("This is a test
\t\tfor move next word
third line");
    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);
    test_view.update_text(true);

    test_view.move_cursor_end_file();
    assert_eq!(3, test_view.cursor.line);
    assert_eq!(10, test_view.cursor.col);

    // third line
    test_view.move_prev_word();
    assert_eq!(7, test_view.cursor.col);
    test_view.move_prev_word();
    assert_eq!(1, test_view.cursor.col);

    // second line
    test_view.move_prev_word();
    assert_eq!(2, test_view.cursor.line);
    assert_eq!(23, test_view.cursor.col);
    test_view.move_prev_word();
    assert_eq!(18, test_view.cursor.col);
    test_view.move_prev_word();
    assert_eq!(13, test_view.cursor.col);
    test_view.move_prev_word();
    assert_eq!(9, test_view.cursor.col);
    test_view.move_prev_word();
    test_view.move_prev_word();

    // first line
    test_view.move_prev_word();
    assert_eq!(1, test_view.cursor.line);
    assert_eq!(11, test_view.cursor.col);
    test_view.move_prev_word();
    assert_eq!(9, test_view.cursor.col);
    test_view.move_prev_word();
    assert_eq!(6, test_view.cursor.col);
    test_view.move_prev_word();
    assert_eq!(1, test_view.cursor.col);
}

#[test]
fn test_delete_end_of_line() {
    let text = String::from("This is a test
\t\tfor delete end of line");
    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);
    test_view.update_text(true);

    // Delete end of line after first word
    test_view.move_next_word();
    test_view.delete_end_of_line();

    assert_eq!(test_view.text,
               "This i
\t\tfor delete end of line\n");
    assert_eq!(6, test_view.cur_line_len);
    // Cursor and text column are the same when the line doesn't have tabs
    assert_eq!(test_view.cursor.col, test_view.cur_line_len);
    assert_eq!(test_view.text_col, test_view.cur_line_len);

    // Switch to Edit mode
    // Delete end of second line after 'f'
    test_view.mode = Mode::Edit;
    test_view.move_cursor(Action::MoveDown);
    test_view.move_cursor_start_line();
    test_view.move_cursor_right();
    test_view.move_cursor_right();

    test_view.delete_end_of_line();
    assert_eq!(test_view.text,
               "This i
\t\tf\n");
    // Text column + 1 because we are in Edit Mode
    assert_eq!(test_view.text_col + 1, test_view.cur_line_len);
}

#[test]
fn test_copy_paste() {
    let text = String::from("\t\t_test for copy and paste
middle line
last line_");
    let name = String::from("name");
    let actions = keyconfig::new();
    let colors = Colors::new();
    let mut test_view = Viewer::new("",
                                    name,
                                    actions,
                                    colors,
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);
    test_view.update_text(true);

    test_view.mode = Mode::Edit;
    test_view.move_cursor_right();
    test_view.move_cursor_right();
    test_view.move_cursor_right();
    test_view.select_copy_start_marker();

    test_view.move_cursor_end_file();
    test_view.move_cursor_left();
    test_view.move_cursor_left();
    test_view.select_copy_end_marker();
    test_view.move_cursor_end_file();

    test_view.add_char('\n');
    test_view.paste();

    assert_eq!("\t\t_test for copy and paste
middle line
last line_
test for copy and paste
middle line
last line\n", test_view.text);
}

#[test]
fn test_add_char() {
    let text = String::from("New test text");
    let mut test_view = Viewer::new("",
                                    String::from("name"),
                                    keyconfig::new(),
                                    Colors::new(),
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);
    test_view.switch_mode(Action::EditMode);
    test_view.update_text(true);

    assert_eq!("New test text\n", test_view.text);
    assert_eq!(14, test_view.cur_line_len);

    test_view.add_char('_');
    assert_eq!("_New test text\n", test_view.text);
    assert_eq!(15, test_view.cur_line_len);

    test_view.move_cursor_right();
    test_view.move_cursor_right();
    test_view.add_char('_');
    assert_eq!("_Ne_w test text\n", test_view.text);
    assert_eq!(16, test_view.cur_line_len);

    test_view.move_cursor_end_line();
    test_view.add_char('_');
    assert_eq!("_Ne_w test text_\n", test_view.text);
}

#[test]
fn test_add_tab() {
    let text = String::from("New test text");
    let mut test_view = Viewer::new("",
                                    String::from("name"),
                                    keyconfig::new(),
                                    Colors::new(),
                                    false,
                                    false,
                                    DEFAULT_TAB_SPACES);
    test_view.model.change_text_for_tests(text);
    test_view.update_text(true);

    test_view.switch_mode(Action::EditMode);
    assert_eq!("New test text\n", test_view.text);
    assert_eq!(14, test_view.cur_line_len);

    test_view.add_tab_spaces();
    assert_eq!("    New test text\n", test_view.text);
    assert_eq!(18, test_view.cur_line_len);

    test_view.move_cursor_right();
    test_view.move_cursor_right();
    test_view.add_tab_spaces();
    assert_eq!("    Ne    w test text\n", test_view.text);
    assert_eq!(22, test_view.cur_line_len);

    test_view.move_cursor_end_line();
    test_view.add_tab_spaces();
    assert_eq!("    Ne    w test text    \n", test_view.text);
    assert_eq!(26, test_view.cur_line_len);
}
