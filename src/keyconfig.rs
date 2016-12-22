use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use rustbox::Key;
use viewer;

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
    };
);

pub fn fill_key_map(filepath: &str) -> HashMap<Key, viewer::Action> {
    // Defaults
    let mut actions = map!{
        Key::Right => viewer::Action::MoveRight,
        Key::Left => viewer::Action::MoveLeft,
        Key::Down => viewer::Action::MoveDown,
        Key::Up => viewer::Action::MoveUp,
        Key::PageUp => viewer::Action::MovePageUp,
        Key::PageDown => viewer::Action::MovePageDown,
        Key::Home => viewer::Action::MoveStartLine,
        Key::End => viewer::Action::MoveEndLine,
        Key::Char('g') => viewer::Action::GoToLine,
        Key::Char('/') => viewer::Action::Search,
        Key::Char('n') => viewer::Action::SearchNext,
        Key::Char('p') => viewer::Action::SearchPrevious,
        Key::Char('w') => viewer::Action::MoveNextWord,
        Key::Char('s') => viewer::Action::MovePrevWord,
        Key::Enter => viewer::Action::Go,
        Key::Char('q') => viewer::Action::Quit
    };

    // Load config file key settings
    let mut config_file = match File::open(filepath) {
        Ok(file) => file,
        Err(_) => {
            info!("Config file {} doesn't exist", filepath);
            return actions;
        }
    };
    info!("Opening config file: {}", filepath);

    let mut text = String::new();
    match config_file.read_to_string(&mut text) {
        Ok(_) => {}
        Err(_) => {
            info!("Error reading config file: {}", filepath);
            return actions;
        }
    }

    for ln in text.lines() {
        if ln.is_empty() || &ln[0..1] == "#" {
            continue;
        }
        let mut split = ln.split(':');
        let key = split.next().unwrap_or("NoKey");
        let k = match key.trim().len() {
            1 => Key::Char(key.trim().chars().next().unwrap()),
            2 => {
                if key == "Up" {
                    Key::Up
                } else {
                    continue;
                }
            }
            3 => {
                if key == "Tab" {
                    Key::Tab
                } else if key == "Esc" {
                    Key::Esc
                } else if key == "End" {
                    Key::End
                } else if key.starts_with("f+") {
                    let (_, n) = key.split_at(2);
                    Key::F(n.parse::<u32>().unwrap())
                } else {
                    continue;
                }
            }
            4 => {
                if key == "Left" {
                    Key::Left
                } else if key == "Down" {
                    Key::Down
                } else if key == "Home" {
                    Key::Home
                } else if key.starts_with("f+") {
                    let (_, n) = key.split_at(2);
                    Key::F(n.parse::<u32>().unwrap())
                } else {
                    continue;
                }
            }
            5 => {
                if key == "Enter" {
                    Key::Enter
                } else if key == "Key::Right" {
                    Key::Right
                } else {
                    continue;
                }
            }
            6 => {
                if key == "Delete" {
                    Key::Delete
                } else if key == "Insert" {
                    Key::Insert
                } else if key == "PageUp" {
                    Key::PageUp
                } else if key.starts_with("ctrl+") {
                    let (_, c) = key.split_at(5);
                    Key::Ctrl(c.chars().next().unwrap())
                } else {
                    continue;
                }
            }
            8 => {
                if key == "PageDown" {
                    Key::PageDown
                } else {
                    continue;
                }
            }
            9 => {
                if key == "Backspace" {
                    Key::Backspace
                } else {
                    continue;
                }
            }
            _ => {
                continue;
            }
        };

        let act = split.next().unwrap_or("NoAct");
        let a = match act.trim() {
            "None" => viewer::Action::None,
            "Go" => viewer::Action::Go,
            "MoveRight" => viewer::Action::MoveRight,
            "MoveLeft" => viewer::Action::MoveLeft,
            "MoveDown" => viewer::Action::MoveDown,
            "MoveUp" => viewer::Action::MoveUp,
            "MovePageUp" => viewer::Action::MovePageUp,
            "MovePageDown" => viewer::Action::MovePageDown,
            "MoveStartLine" => viewer::Action::MoveStartLine,
            "MoveEndLine" => viewer::Action::MoveEndLine,
            "GoToLine" => viewer::Action::GoToLine,
            "Search" => viewer::Action::Search,
            "SearchNext" => viewer::Action::SearchNext,
            "SearchPrevious" => viewer::Action::SearchPrevious,
            "MoveNextWord" => viewer::Action::MoveNextWord,
            "MovePrevWord" => viewer::Action::MovePrevWord,
            "Quit" => viewer::Action::Quit,
            _ => {
                continue;
            }
        };

        actions.insert(k, a);
    }

    actions
}
