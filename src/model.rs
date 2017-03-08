// Model

use std::error::Error as StdError;

use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

mod errors {}

pub struct Model {
    text: String,
    text_vec: Vec<String>,
    old_text: String,
    line_count: usize,
    filepath: String,
    saved: bool,
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

        let mut text_vec: Vec<String> = Vec::new();

        if filepath != "" {
            text = open_file(filepath);
            line_count = text.lines().count();
        } else {
            // Empty path for test cases
            text = String::new();
            line_count = 0;
        }

        for line in text.lines() {
            text_vec.push(String::from(line));
        }

        Model {
            text: text,
            text_vec: text_vec,
            old_text: String::new(),
            filepath: filepath.to_string(),
            line_count: line_count,
            saved: true,
        }
    }

    pub fn get_text(&self) -> String {
        let mut text = String::new();

        for line in &self.text_vec {
            text.push_str(line);
            text.push('\n');
        }

        text
    }

    #[allow(dead_code)]
    pub fn get_text_slice(&self, start_line: usize, amount: usize) -> String {
        let mut slice = String::new();

        if start_line <= self.line_count {
            let mut lines = self.text.lines().skip(start_line - 1);
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
        if let Some(l) = self.text.lines().nth(line - 1) {
            if let Some(c) = l.chars().nth(column - 1) {
                return c;
            }
        }

        info!("Out of range in get_char() {}:{}", line, column);
        '_'
    }

    pub fn get_line(&self, line: usize) -> String {
        if line <= self.text_vec.len() {
            return self.text_vec[line - 1].clone();
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
        self.text_vec.len()
    }

    pub fn get_saved_stat(&self) -> bool {
        self.saved
    }

    pub fn add_char(&mut self, c: char, line: usize, column: usize) {
        // TODO: Use better data structure for strings. For example, a Rope
        let mut new_text = String::new();
        let mut tmp_line = String::new();

        let line_clone = self.text_vec[line - 1].clone();
        let (beg, end) = line_clone.split_at(column - 1);

        if c != '\n' {
            tmp_line.push_str(&format!("{}{}{}", beg, c, end));

            self.text_vec[line - 1] = tmp_line;
        } else {
            if line == self.text_vec.len() &&
               (column + 1) == self.text_vec[line - 1].len() {
                self.text_vec.push(tmp_line);
            } else {
                self.text_vec[line - 1] = String::from(beg);
                self.text_vec.insert(line, String::from(end));
            }
            self.line_count += 1;
        }

        for ln in &self.text_vec {
            new_text.push_str(ln.as_ref());
            new_text.push('\n');
        }
        self.old_text = self.text.clone();
        self.text = new_text;
        self.saved = false;
    }

    pub fn add_block(&mut self, copy_str: String, line: usize, column: usize) {
        // TODO: Too similar to add_char(), overload?
        if line > self.text.lines().count() {
            warn!("line parameter is past the end of file");
            return;
        }

        if column - 1 > self.text_vec[line - 1].len() {
            warn!("column parameter is past line text");
            return;
        }

        let mut new_text = String::new();
        let mut tmp_line = String::new();

        let line_clone = self.text_vec[line - 1].clone();
        let (beg, end) = line_clone.split_at(column - 1);

        tmp_line.push_str(&format!("{}{}{}", beg, copy_str, end));
        self.text_vec[line - 1] = tmp_line;

        for ln in &self.text_vec {
            new_text.push_str(ln.as_ref());
            new_text.push('\n');
        }

        self.line_count += copy_str.lines().count() - 1;

        self.old_text = self.text.clone();
        self.text = new_text;
        self.saved = false;
    }

    pub fn delete_char(&mut self, line: usize, column: usize) -> usize {
        // TODO: Use better data structure for strings. For example, a Rope
        // Can't delete from the beginning of the file
        if line == 1 && column == 1 {
            return 0;
        }

        let mut new_text = String::new();
        let mut tmp_line = String::new();
        let mut end_len = 0;

        let line_clone = self.text_vec[line - 1].clone();
        let (tmp_beg, tmp_end) = line_clone.split_at(column - 1);
        let mut beg = tmp_beg.to_string();
        let end = tmp_end.to_string();

        if beg.is_empty() {
            tmp_line.push_str(&format!("{}{}", self.text_vec[line - 2], line_clone));
            self.text_vec[line - 2] = tmp_line;
            self.text_vec.remove(line - 1);
            end_len = self.text_vec[line - 2].len();
        } else {
            beg.pop();
            tmp_line.push_str(&format!("{}{}", beg, end));
            self.text_vec[line - 1] = tmp_line;
        }

        for ln in &self.text_vec {
            new_text.push_str(ln.as_ref());
            new_text.push('\n');
        }

        self.old_text = self.text.clone();
        self.text = new_text;
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
        if column <= chars || line > self.text_vec.len() {
            return;
        }

        info!("Deleting {} chars from {}:{}",
              chars,
              line,
              column);

        let mut new_text = String::new();
        let mut tmp_line = String::new();

        let line_clone = self.text_vec[line - 1].clone();
        let (tmp_beg, tmp_end) = line_clone.split_at(column - 1);
        let beg = tmp_beg[0..(tmp_beg.len() - chars)].to_string();
        let end = tmp_end.to_string();
        tmp_line.push_str(&format!("{}{}", beg, end));
        self.text_vec[line - 1] = tmp_line;

        for ln in &self.text_vec {
            new_text.push_str(ln.as_ref());
            new_text.push('\n');
        }

        self.old_text = self.text.clone();
        self.text = new_text;
        self.saved = false;
    }

    pub fn delete_line(&mut self, line: usize) -> bool {
        // TODO: Can't delete only line in the file
        info!("Delete line {}", line);
        if self.line_count == 1 || line > self.line_count {
            return false;
        }

        let mut new_text = String::new();
        self.text_vec.remove(line - 1);

        for ln in &self.text_vec {
            new_text.push_str(ln.as_ref());
            new_text.push('\n');
        }

        self.old_text = self.text.clone();
        self.text = new_text;
        self.saved = false;
        self.line_count -= 1;

        true
    }

    pub fn undo(&mut self) {
        // TODO: Optimize. For example with a stack of changes
        // This only reverts back one change :(
        self.text = self.old_text.clone();

        self.text_vec.clear();
        for line in self.old_text.lines() {
            self.text_vec.push(String::from(line));
        }

        self.line_count = self.text.lines().count();
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

        match file.write_all(self.text.as_bytes()) {
            Ok(_) => {}
            Err(error) => {
                error!("Couldn't write to {} because {}",
                       path.display(),
                       error);
                return;
            }
        }
        match file.set_len(self.text.len() as u64) {
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

        self.text_vec.clear();
        for line in text.lines() {
            self.text_vec.push(String::from(line));
        }

        self.text = text;
    }
}
