use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use rustbox::Key;
use viewer;

pub fn fill_key_map(filepath: &str) -> HashMap<Key, viewer::Action> {
    // Defaults
    let mut actions = HashMap::new();
    actions.insert(Key::Right, viewer::Action::MoveRight);
    actions.insert(Key::Left, viewer::Action::MoveLeft);
    actions.insert(Key::Down, viewer::Action::MoveDown);
    actions.insert(Key::Up, viewer::Action::MoveUp);
    actions.insert(Key::PageUp, viewer::Action::MovePageUp);
    actions.insert(Key::PageDown, viewer::Action::MovePageDown);
    actions.insert(Key::Home, viewer::Action::MoveStartLine);
    actions.insert(Key::End, viewer::Action::MoveEndLine);
    actions.insert(Key::Char('g'), viewer::Action::GoToLine);
    actions.insert(Key::Enter, viewer::Action::Go);
    actions.insert(Key::Char('q'), viewer::Action::Quit);

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
        if ln.len() == 0 || &ln[0..1] == "#" {
            continue;
        }
        let mut split = ln.split(':');
        let key = split.next().unwrap_or("NoKey");
        info!("len of key {}", key.trim().len());
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
            "MoveRight" => viewer::Action::MoveRight,
            "MoveLeft" => viewer::Action::MoveLeft,
            "MoveDown" => viewer::Action::MoveDown,
            "MoveUp" => viewer::Action::MoveUp,
            "MovePageUp" => viewer::Action::MovePageUp,
            "MovePageDown" => viewer::Action::MovePageDown,
            "MoveStartLine" => viewer::Action::MoveStartLine,
            "MoveEndLine" => viewer::Action::MoveEndLine,
            "GoToLine" => viewer::Action::GoToLine,
            "Quit" => viewer::Action::Quit,
            _ => {
                continue;
            }
        };

        actions.insert(k, a);
    }

    actions
}
