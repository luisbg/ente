// Key Configuration

use rustbox::Key;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
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

pub fn new() -> HashMap<Key, viewer::Action> {
    // Defaults
    map!{
        Key::Right => viewer::Action::MoveRight,
        Key::Left => viewer::Action::MoveLeft,
        Key::Down => viewer::Action::MoveDown,
        Key::Up => viewer::Action::MoveUp,
        Key::PageUp => viewer::Action::MovePageUp,
        Key::PageDown => viewer::Action::MovePageDown,
        Key::Home => viewer::Action::MoveStartLine,
        Key::End => viewer::Action::MoveEndLine,
        Key::Char('<') => viewer::Action::MoveStartFile,
        Key::Char('>') => viewer::Action::MoveEndFile,
        Key::Char('e') => viewer::Action::EditMode,
        Key::Char('a') => viewer::Action::Append,
        Key::Ctrl('r') => viewer::Action::ReadMode,
        Key::Char('g') => viewer::Action::GoToLine,
        Key::Char('/') => viewer::Action::Search,
        Key::Char('n') => viewer::Action::SearchNext,
        Key::Char('p') => viewer::Action::SearchPrevious,
        Key::Char('w') => viewer::Action::MoveNextWord,
        Key::Char('s') => viewer::Action::MovePrevWord,
        Key::Ctrl('d') => viewer::Action::KillLine,
        Key::Ctrl('k') => viewer::Action::KillEndLine,
        Key::Delete => viewer::Action::Delete,
        Key::Ctrl('x') => viewer::Action::CopyStartMark,
        Key::Ctrl('c') => viewer::Action::CopyEndMark,
        Key::Ctrl('v') => viewer::Action::Paste,
        Key::Ctrl('z') => viewer::Action::Undo,
        Key::Ctrl('l') => viewer::Action::ToggleLineNumbers,
        Key::Enter => viewer::Action::Go,
        Key::Ctrl('s') => viewer::Action::Save,
        Key::Ctrl('q') => viewer::Action::Quit,
        Key::F(1) => viewer::Action::Help
    }
}

pub fn fill_key_map(filepath: &str) -> HashMap<Key, viewer::Action> {
    let mut actions = new();

    // Load config file key settings
    let path = Path::new(filepath);
    if path.is_dir() {
        info!("Key config file can't be a folder. {}", filepath);
        return actions;
    }

    if !path.is_file() {
        info!("Key config file {} doesn't exist", filepath);
        return actions;
    }

    let mut config_file = match File::open(path) {
        Ok(file) => {
            info!("Opening key config file: {}", filepath);
            file
        }
        Err(_) => {
            info!("Error opening key config file {}", filepath);
            return actions;
        }
    };

    let mut text = String::new();
    match config_file.read_to_string(&mut text) {
        Ok(_) => {}
        Err(_) => {
            info!("Error reading config file: {}", filepath);
            return actions;
        }
    }

    parse_config_file(text, &mut actions);

    actions
}

fn parse_config_file(text: String,
                     actions: &mut HashMap<Key, viewer::Action>) {
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
            "MoveStartFile" => viewer::Action::MoveStartFile,
            "MoveEndFile" => viewer::Action::MoveEndFile,
            "EditMode" => viewer::Action::EditMode,
            "ReadMode" => viewer::Action::ReadMode,
            "Append" => viewer::Action::Append,
            "GoToLine" => viewer::Action::GoToLine,
            "Search" => viewer::Action::Search,
            "SearchNext" => viewer::Action::SearchNext,
            "SearchPrevious" => viewer::Action::SearchPrevious,
            "MoveNextWord" => viewer::Action::MoveNextWord,
            "MovePrevWord" => viewer::Action::MovePrevWord,
            "KillLine" => viewer::Action::KillLine,
            "KillEndLine" => viewer::Action::KillEndLine,
            "Delete" => viewer::Action::Delete,
            "CopyStartMark" => viewer::Action::CopyStartMark,
            "CopyEndMark" => viewer::Action::CopyEndMark,
            "Paste" => viewer::Action::Paste,
            "Undo" => viewer::Action::Undo,
            "ToggleLineNumbers" => viewer::Action::ToggleLineNumbers,
            "Save" => viewer::Action::Save,
            "Quit" => viewer::Action::Quit,
            "Help" => viewer::Action::Help,
            _ => {
                continue;
            }
        };

        info!("Adding key {:?} to match action {:?}", k, a);
        actions.insert(k, a);
    }
}

#[test]
fn test_default_keys() {
    let map = new();

    assert_eq!(map.get(&Key::Right).unwrap(), &viewer::Action::MoveRight);
    assert_eq!(map.get(&Key::Left).unwrap(), &viewer::Action::MoveLeft);
    assert_eq!(map.get(&Key::Down).unwrap(), &viewer::Action::MoveDown);
    assert_eq!(map.get(&Key::Up).unwrap(), &viewer::Action::MoveUp);
    assert_eq!(map.get(&Key::PageUp).unwrap(), &viewer::Action::MovePageUp);
    assert_eq!(map.get(&Key::PageDown).unwrap(), &viewer::Action::MovePageDown);
    assert_eq!(map.get(&Key::Home).unwrap(), &viewer::Action::MoveStartLine);
    assert_eq!(map.get(&Key::End).unwrap(), &viewer::Action::MoveEndLine);

    assert_eq!(map.get(&Key::Char('<')).unwrap(),
               &viewer::Action::MoveStartFile);
    assert_eq!(map.get(&Key::Char('>')).unwrap(), &viewer::Action::MoveEndFile);
    assert_eq!(map.get(&Key::Char('e')).unwrap(), &viewer::Action::EditMode);
    assert_eq!(map.get(&Key::Char('a')).unwrap(), &viewer::Action::Append);
    assert_eq!(map.get(&Key::Ctrl('r')).unwrap(), &viewer::Action::ReadMode);

    assert_eq!(map.get(&Key::Char('g')).unwrap(), &viewer::Action::GoToLine);
    assert_eq!(map.get(&Key::Char('/')).unwrap(), &viewer::Action::Search);
    assert_eq!(map.get(&Key::Char('n')).unwrap(), &viewer::Action::SearchNext);
    assert_eq!(map.get(&Key::Char('p')).unwrap(),
               &viewer::Action::SearchPrevious);
    assert_eq!(map.get(&Key::Char('w')).unwrap(),
               &viewer::Action::MoveNextWord);
    assert_eq!(map.get(&Key::Char('s')).unwrap(),
               &viewer::Action::MovePrevWord);
    assert_eq!(map.get(&Key::Ctrl('d')).unwrap(), &viewer::Action::KillLine);
    assert_eq!(map.get(&Key::Ctrl('k')).unwrap(), &viewer::Action::KillEndLine);

    assert_eq!(map.get(&Key::Delete).unwrap(), &viewer::Action::Delete);
    assert_eq!(map.get(&Key::Ctrl('x')).unwrap(),
               &viewer::Action::CopyStartMark);
    assert_eq!(map.get(&Key::Ctrl('c')).unwrap(), &viewer::Action::CopyEndMark);
    assert_eq!(map.get(&Key::Ctrl('v')).unwrap(), &viewer::Action::Paste);
    assert_eq!(map.get(&Key::Ctrl('z')).unwrap(), &viewer::Action::Undo);
    assert_eq!(map.get(&Key::Ctrl('l')).unwrap(),
               &viewer::Action::ToggleLineNumbers);
    assert_eq!(map.get(&Key::Enter).unwrap(), &viewer::Action::Go);
    assert_eq!(map.get(&Key::Ctrl('s')).unwrap(), &viewer::Action::Save);
    assert_eq!(map.get(&Key::Ctrl('q')).unwrap(), &viewer::Action::Quit);

    assert_eq!(map.get(&Key::F(1)).unwrap(), &viewer::Action::Help);
}

#[test]
fn test_fill_keys() {
    let mut map = new();
    let text = String::from("\
ctrl+p: MovePageUp
ctrl+n: MovePageDown
x: Delete
Esc: Quit
f+2: Save
    ");

    parse_config_file(text, &mut map);

    assert_eq!(map.get(&Key::Ctrl('p')).unwrap(), &viewer::Action::MovePageUp);
    assert_eq!(map.get(&Key::Ctrl('n')).unwrap(),
               &viewer::Action::MovePageDown);
    assert_eq!(map.get(&Key::Char('x')).unwrap(), &viewer::Action::Delete);
    assert_eq!(map.get(&Key::Esc).unwrap(), &viewer::Action::Quit);
    assert_eq!(map.get(&Key::F(2)).unwrap(), &viewer::Action::Save);
}
