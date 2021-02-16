extern crate termion;

use termion::{color, style};

pub fn bold(text: &str) -> String {
    return format!("{}{}{}", style::Bold, text, style::Reset);
}
pub fn yellow(text: &str) -> String {
    let yellow = color::Fg(color::Yellow);
    let reset = color::Fg(color::Reset);
    return format!("{}{}{}", yellow, text, reset);
}

pub fn emphasize(text: &str) -> String {
    return bold(&yellow(text));
}
