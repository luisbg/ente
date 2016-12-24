mod errors {}

pub struct Model {
    text: String,
}

impl Model {
    pub fn new(text: &str) -> Model {
        let model = Model {
            text: String::from(text),
        };

        model
    }

    pub fn get_text(&mut self) -> String {
        self.text.clone()
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

        end_len
    }
}
