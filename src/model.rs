// Model

use std::error::Error as StdError;

use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

mod errors {}

pub struct Model {
    text: Vec<String>,
    old_text: Vec<String>,
    line_count: usize,
    filepath: String,
    saved: bool,
}

pub struct Position {
    pub line: usize,
    pub col: usize,
}

fn open_file(filepath: &str) -> String {
    // Open the file
    let mut text = String::new();

    let path = Path::new(filepath);
    if path.is_dir() {
        panic!("Can't open a folder. {}", filepath);
    }

    if path.is_file() {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => panic!("File {} does not exist. Creating it.", filepath),
        };

        info!("Opening file: {}", filepath);

        // Read the file into a String
        match file.read_to_string(&mut text) {
            Ok(_) => {}
            Err(error) => panic!("couldn't read {}: ", error.description()),
        }
    } else {
        info!("Creating new file: {}", filepath);
        File::create(path).expect("Couldn't create file");
    }

    if text.lines().count() == 0 {
        text.push('\n');
    }

    text
}

impl Model {
    pub fn new(filepath: &str) -> Model {
        let text: String;
        let line_count: usize;

        let mut model_text: Vec<String> = Vec::new();

        if filepath != "" {
            text = open_file(filepath);
            line_count = text.lines().count();
        } else {
            // Empty path for test cases
            text = String::new();
            line_count = 0;
        }

        for line in text.lines() {
            model_text.push(String::from(line));
        }

        let old_text = model_text.clone();

        Model {
            text: model_text,
            old_text: old_text,
            filepath: filepath.to_string(),
            line_count: line_count,
            saved: true,
        }
    }

    #[allow(dead_code)]
    pub fn get_text(&self) -> String {
        let mut text = String::new();

        for line in &self.text {
            text.push_str(line);
            text.push('\n');
        }

        text
    }

    #[allow(dead_code)]
    pub fn get_text_slice(&self, start_line: usize, amount: usize) -> String {
        let mut slice = String::new();

        if start_line <= self.line_count {
            let mut lines = self.text.iter().skip(start_line - 1);
            for _ in 0..amount {
                if let Some(l) = lines.next() {
                    slice.push_str(l);
                    slice.push('\n');
                }
            }
        } else {
            info!("Out of range in get_text_slice() {}", start_line);
        }

        slice
    }

    #[allow(dead_code)]
    pub fn get_char(&self, line: usize, column: usize) -> char {
        if let Some(l) = self.text.iter().nth(line - 1) {
            if let Some(c) = l.chars().nth(column - 1) {
                return c;
            }
        }

        info!("Out of range in get_char() {}:{}", line, column);
        '_'
    }

    pub fn get_line(&self, line: usize) -> String {
        if line <= self.text.len() {
            return self.text[line - 1].clone();
        }

        info!("Out of range in get_line() {}", line);
        String::new()
    }

    #[allow(dead_code)]
    pub fn get_line_len(&self, line: usize) -> usize {
        let ln = self.get_line(line);
        ln.len()
    }

    pub fn get_line_count(&self) -> usize {
        self.text.len()
    }

    pub fn get_saved_stat(&self) -> bool {
        self.saved
    }

    pub fn add_char(&mut self, c: char, line: usize, column: usize) {
        // TODO: Use better data structure for strings. For example, a Rope
        let mut tmp_line = String::new();

        self.old_text = self.text.clone();

        let line_clone = self.text[line - 1].clone();
        let (beg, end) = line_clone.split_at(column - 1);

        if c != '\n' {
            tmp_line.push_str(&format!("{}{}{}", beg, c, end));

            self.text[line - 1] = tmp_line;
        } else {
            if line == self.text.len() &&
               (column + 1) == self.text[line - 1].len() {
                self.text.push(tmp_line);
            } else {
                self.text[line - 1] = String::from(beg);
                self.text.insert(line, String::from(end));
            }
            self.line_count += 1;
        }

        self.saved = false;
    }

    pub fn add_block(&mut self, copy_str: String, line: usize, column: usize) {
        // TODO: Too similar to add_char(), overload?
        if line > self.text.len() {
            warn!("line parameter is past the end of file");
            return;
        }

        if column - 1 > self.text[line - 1].len() {
            warn!("column parameter is past line text");
            return;
        }

        self.old_text = self.text.clone();

        let mut tmp_line = String::new();

        let line_clone = self.text[line - 1].clone();
        let (beg, end) = line_clone.split_at(column - 1);

        tmp_line.push_str(&format!("{}{}{}", beg, copy_str, end));
        self.text[line - 1] = tmp_line;

        self.line_count += copy_str.lines().count() - 1;
        self.saved = false;
    }

    pub fn delete_char(&mut self, line: usize, column: usize) -> usize {
        // TODO: Use better data structure for strings. For example, a Rope
        // Can't delete from the beginning of the file
        if line == 1 && column == 1 {
            return 0;
        }

        self.old_text = self.text.clone();

        let mut tmp_line = String::new();
        let mut end_len = 0;

        let line_clone = self.text[line - 1].clone();
        let (tmp_beg, tmp_end) = line_clone.split_at(column - 1);
        let mut beg = tmp_beg.to_string();
        let end = tmp_end.to_string();

        if beg.is_empty() {
            tmp_line.push_str(&format!("{}{}",
                                       self.text[line - 2],
                                       line_clone));
            self.text[line - 2] = tmp_line;
            self.text.remove(line - 1);
            end_len = self.text[line - 2].len();
        } else {
            beg.pop();
            tmp_line.push_str(&format!("{}{}", beg, end));
            self.text[line - 1] = tmp_line;
        }

        self.saved = false;

        if column == 1 {
            self.line_count -= 1;
        }

        end_len
    }

    pub fn delete_block(&mut self, line: usize, column: usize, chars: usize) {
        if chars == 0 {
            return;
        }
        // There needs to be enough chars left of the cursor in the line
        if column <= chars || line > self.text.len() {
            return;
        }

        info!("Deleting {} chars from {}:{}",
              chars,
              line,
              column);

        self.old_text = self.text.clone();

        let mut tmp_line = String::new();

        let line_clone = self.text[line - 1].clone();
        let (tmp_beg, tmp_end) = line_clone.split_at(column - 1);
        let beg = tmp_beg[0..(tmp_beg.len() - chars)].to_string();
        let end = tmp_end.to_string();
        tmp_line.push_str(&format!("{}{}", beg, end));
        self.text[line - 1] = tmp_line;

        self.saved = false;
    }

    pub fn delete_line(&mut self, line: usize) -> bool {
        // TODO: Can't delete only line in the file
        info!("Delete line {}", line);
        if self.line_count == 1 || line > self.line_count {
            return false;
        }

        self.old_text = self.text.clone();

        self.text.remove(line - 1);
        self.saved = false;
        self.line_count -= 1;

        true
    }

    pub fn forward_search(&mut self,
                          search_str: &str,
                          cur_pos: Position)
                          -> Position {
        let mut lines = self.text.iter().skip(cur_pos.line - 1);
        let mut line_num = 0;
        let mut col = 0;

        // Check current line after the cursor
        let (_, rest_line) = lines.next().unwrap().split_at(cur_pos.col);
        match rest_line.find(search_str) {
            Some(c) => {
                line_num = cur_pos.line;
                col = c + cur_pos.col + 1;
            }
            None => {
                // If nothing found in current line, search in the rest
                for ln in cur_pos.line..self.text.len() {
                    match lines.next() {
                        Some(l) => {
                            if let Some(c) = l.find(search_str) {
                                line_num = ln + 1;
                                col = c + 1;
                                break; // Found it
                            }
                        }
                        _ => {
                            return Position { line: 0, col: 0 };
                        }
                    }
                }
            }
        }

        Position {
            line: line_num,
            col: col,
        }
    }

    pub fn backward_search(&mut self,
                           search_str: &str,
                           cur_pos: Position)
                           -> Position {
        let mut lines = self.text
            .iter()
            .rev()
            .skip(self.get_line_count() - cur_pos.line);
        let mut line_num = 0;
        let mut col = 0;

        // Check current line before the cursor
        let (beg_line, _) = lines.next().unwrap().split_at(cur_pos.col);
        match beg_line.rfind(search_str) {
            Some(c) => {
                line_num = cur_pos.line;
                col = c + 1;
            }
            None => {
                // If nothing found in current line, search in the rest
                for ln in (1..cur_pos.line).rev() {
                    match lines.next() {
                        Some(l) => {
                            if let Some(c) = l.rfind(search_str) {
                                line_num = ln;
                                col = c + 1;
                                break; // Found it
                            }
                        }
                        _ => {
                            return Position { line: 0, col: 0 };
                        }
                    }
                }
            }
        }

        Position {
            line: line_num,
            col: col,
        }
    }

    pub fn undo(&mut self) {
        // TODO: Optimize. For example with a stack of changes
        // This only reverts back one change :(
        self.text.clear();
        self.text = self.old_text.clone();
        self.line_count = self.text.len();
        self.saved = false;
    }

    pub fn save(&mut self) {
        let path = Path::new(&self.filepath);
        let mut file = match OpenOptions::new().write(true).open(&path) {
            Ok(file) => file,
            Err(error) => {
                error!("There was a problem opening the file: {}", error);
                return;
            }
        };

        let mut text = String::new();
        for ln in &self.text {
            text.push_str(ln.as_ref());
            text.push('\n');
        }

        match file.write_all(text.as_bytes()) {
            Ok(_) => {}
            Err(error) => {
                error!("Couldn't write to {} because {}",
                       path.display(),
                       error);
                return;
            }
        }
        match file.set_len(text.len() as u64) {
            Ok(_) => info!("Successfully saved file: {}", path.display()),
            Err(error) => {
                error!("Couldn't truncate file {} because {}",
                       path.display(),
                       error);
                return;
            }
        }

        self.saved = true;
    }

    #[allow(dead_code)]
    pub fn change_text_for_tests(&mut self, text: String) {
        self.line_count = text.lines().count();

        self.text.clear();
        for line in text.lines() {
            self.text.push(String::from(line));
        }
    }
}
