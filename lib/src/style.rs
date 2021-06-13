extern crate termion;

use termion::{color, style};

pub fn bold(text: &str) -> String {
    format!("{}{}{}", style::Bold, text, style::Reset)
}
pub fn yellow(text: &str) -> String {
    let yellow = color::Fg(color::Yellow);
    let reset = color::Fg(color::Reset);
    format!("{}{}{}", yellow, text, reset)
}
pub fn red<T: std::fmt::Display>(text: T) -> String {
    let red = color::Fg(color::Red);
    let reset = color::Fg(color::Reset);
    format!("{}{}{}", red, text, reset)
}

pub fn emphasize(text: &str) -> String {
    bold(&yellow(text))
}
