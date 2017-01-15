use std::fs::File;
use std::io::prelude::*;

use rustbox::Color;
use viewer;

pub fn fill_colors(filepath: &str) -> viewer::Colors {
    // Default colors
    let mut colors = viewer::Colors {
        fg: Color::White,
        bg: Color::Black,
        line_num: Color::Blue,
        error: Color::Red,
    };

    // Load config file for colors
    let mut config_file = match File::open(filepath) {
        Ok(file) => file,
        Err(_) => {
            info!("Config file {} doesn't exist", filepath);
            return colors;
        }
    };
    info!("Opening config file: {}", filepath);

    let mut text = String::new();
    match config_file.read_to_string(&mut text) {
        Ok(_) => {}
        Err(_) => {
            info!("Error reading config file: {}", filepath);
            return colors;
        }
    }

    parse_config_file(text, &mut colors);

    colors
}

fn parse_config_file(text: String, colors: &mut viewer::Colors) {
    for ln in text.lines() {
        if ln.is_empty() || &ln[0..1] == "#" {
            continue;
        }
        let mut split = ln.split(':');
        let key = split.next().unwrap_or("NoItem");
        let k = match key.trim() {
            "foreground" => 1,
            "background" => 2,
            "line_numbers" => 3,
            "errors" => 4,
            _ => {
                continue;
            }
        };

        let color = split.next().unwrap_or("NoColor");
        let c = match color.trim() {
            "Black" => Color::Black,
            "Red" => Color::Red,
            "Green" => Color::Green,
            "Yellow" => Color::Yellow,
            "Blue" => Color::Blue,
            "Magenta" => Color::Magenta,
            "Cyan" => Color::Cyan,
            "White" => Color::White,
            _ => {
                continue;
            }
        };

        match k {
            1 => {
                colors.fg = c;
            }
            2 => {
                colors.bg = c;
            }
            3 => {
                colors.line_num = c;
            }
            4 => {
                colors.error = c;
            }
            _ => {
                continue;
            }
        }
    }
}
