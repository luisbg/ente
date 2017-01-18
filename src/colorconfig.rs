use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use rustbox::Color;
use viewer;

pub fn fill_colors(filepath: &str) -> viewer::Colors {
    // Default colors
    let mut colors = viewer::Colors::new();

    // Load config file for colors
    let path = Path::new(filepath);
    if path.is_dir() {
        info!("Color config file can't be a folder. {}",
              filepath);
        return colors;
    }

    if !path.is_file() {
        info!("Color config file {} doesn't exist", filepath);
        return colors;
    }

    let mut config_file = match File::open(filepath) {
        Ok(file) => {
            info!("Opening color config file: {}", filepath);
            file
        }
        Err(_) => {
            info!("Error opening color config file {}", filepath);
            return colors;
        }
    };

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
    'color_list: for ln in text.lines() {
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
                let mut parsing_ok = 0;
                let mut colors = Vec::new();

                // We run it as an iterator to avoid assuming a length for the
                // split
                for v in color.trim().split('-') {
                    match v.parse::<u16>() {
                        Ok(value) => {
                            colors.push(value);
                            parsing_ok += 1;
                        }
                        Err(_) => {
                            info!("{} isn't a correct color format",
                                  color.trim());
                            continue 'color_list;
                        }
                    }
                }
                if parsing_ok == 1 {
                    if colors[0] > 255 {
                        info!("{} isn't a correct color format", color.trim());
                        continue 'color_list;
                    }

                    Color::Byte(colors[0])
                } else if parsing_ok == 3 {
                    let color_num = 16 + colors[0] * 36 + colors[1] * 6 +
                                    colors[2];
                    if color_num > 231 {
                        info!("Bad color {}. Each component must be in range \
                               0-5",
                              color_num);
                        continue 'color_list;
                    }

                    Color::Byte(color_num)
                } else {
                    info!("{} isn't a correct color format", color.trim());
                    continue 'color_list;
                }
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

#[test]
fn test_default_colors() {
    let colors = viewer::Colors::new();

    assert_eq!(colors.fg, Color::White);
    assert_eq!(colors.bg, Color::Black);
    assert_eq!(colors.line_num, Color::Blue);
    assert_eq!(colors.error, Color::Red);
}

#[test]
fn test_predefined_colors() {
    let mut colors = viewer::Colors::new();
    let text = String::from("\
        foreground: Green
        background: Yellow
        line_numbers: Magenta
        errors: Cyan
    ");

    parse_config_file(text, &mut colors);

    assert_eq!(colors.fg, Color::Green);
    assert_eq!(colors.bg, Color::Yellow);
    assert_eq!(colors.line_num, Color::Magenta);
    assert_eq!(colors.error, Color::Cyan);
}

#[test]
fn test_rgb_colors() {
    let mut colors = viewer::Colors::new();
    let text = String::from("\
        foreground: 1-2-3
        background: 4-5-0
        line_numbers: 5-4-3
        errors: 2-1-0
    ");

    parse_config_file(text, &mut colors);

    assert_eq!(colors.fg, Color::Byte(67));
    assert_eq!(colors.bg, Color::Byte(190));
    assert_eq!(colors.line_num, Color::Byte(223));
    assert_eq!(colors.error, Color::Byte(94));
}

#[test]
fn test_byte_colors() {
    let mut colors = viewer::Colors::new();
    let text = String::from("\
        foreground: 5
        background: 10
        line_numbers: 232
        errors: 255
    ");

    parse_config_file(text, &mut colors);

    assert_eq!(colors.fg, Color::Byte(5));
    assert_eq!(colors.bg, Color::Byte(10));
    assert_eq!(colors.line_num, Color::Byte(232));
    assert_eq!(colors.error, Color::Byte(255));
}
