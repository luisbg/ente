use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

mod errors {}

pub struct Model {
    text: String,
    line_count: usize,
    filepath: String,
    saved: bool,
}

impl Model {
    pub fn new(text: &str, filepath: &str) -> Model {
        let line_count = text.lines().count();

        Model {
            text: String::from(text),
            filepath: filepath.to_string(),
            line_count: line_count,
            saved: false,
        }
    }

    pub fn get_text(&mut self) -> String {
        self.text.clone()
    }

    pub fn get_line_count(&mut self) -> usize {
        self.line_count
    }

    pub fn get_saved_stat(&mut self) -> bool {
        self.saved
    }

    pub fn add_char(&mut self, c: char, line: usize, column: usize) {
        // TODO: Use better data structure for strings. For example, a Rope
        let mut new_text = String::new();
        for (x, ln) in self.text.lines().enumerate() {
            if x == line - 1 {
                let (beg, end) = ln.split_at(column - 1);
                new_text.push_str(&format!("{}{}{}\n", beg, c, end));
            } else {
                new_text.push_str(ln);
                new_text.push('\n');
            }
        }
        self.text = new_text;
        self.saved = true;

        if c == '\n' {
            self.line_count += 1;
        }
    }

    pub fn delete_char(&mut self, line: usize, column: usize) -> usize {
        // TODO: Use better data structure for strings. For example, a Rope
        // Can't delete from the beginning of the file
        if line == 1 && column == 1 {
            return 0;
        }

        let mut new_text = String::new();
        let mut end_len = 0;
        for (x, ln) in self.text.lines().enumerate() {
            if x == line - 1 {
                let (tmp_beg, tmp_end) = ln.split_at(column - 1);
                let mut beg = tmp_beg.to_string();
                let end = tmp_end.to_string();

                if beg.is_empty() {
                    new_text.pop(); // remove newline from previous line
                    new_text.push_str(&format!("{}{}\n", beg, end));
                    end_len = end.len();
                } else {
                    beg.pop(); // remove character at the cursor
                    new_text.push_str(&format!("{}{}\n", beg, end));
                }
            } else {
                new_text.push_str(ln);
                new_text.push('\n');
            }
        }
        self.text = new_text;
        self.saved = true;

        if column == 1 {
            self.line_count -= 1;
        }

        end_len
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

        self.saved = false;
    }
}