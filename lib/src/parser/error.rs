use crate::{
    style::emphasize,
    token::{Position, TokenType},
};

use crate::style::{bold, yellow};

pub type ParserResult<T> = Result<T, String>;
// All the different types of parsers (like parse_array, parse_match, etc)
#[derive(Debug, Clone)]
pub enum ParserType {
    Assign,
    // Return,
    // Number,
    // Prefix,
    Grouped,
    // Boolean,
    // Identifier,
    Function,
    // Infix,
    Call,
    If,
    Import,

    Match,
    Array,
    Hash,
}
pub enum ParserError<'a> {
    ExpectedFound(&'a TokenType, &'a TokenType, Position),
    // NoPrefixFound(&'a Token),
}

fn assign_msg(err: ParserError) -> String {
    match err {
    ParserError::ExpectedFound(&TokenType::Assign, got, position) => {
      return format!("When parsing an assignment statement on line {}, column {}, we were looking a {}, but we found something else ({:?}). There's a good change you forgot to put an equals sign, or put another token before it.", position.line, position.column, bold(&yellow("=")), got)
    },
    // The wildcard is safe here because this is the only expect_peek
    _ => String::new()
  }
}

fn array_msg(err: ParserError) -> String {
    match err {
        ParserError::ExpectedFound(&TokenType::RightBracket, got, position) => {
            let got = format!("{}", got);
            format!("When parsing an array on line {}, column {}, we were looking for right bracket {}, but we found something else ({}).
    
Hint: Double check to make sure you've closed all your arrays, and you should be on your way", position.line, position.column,emphasize("]"), emphasize(&got))
        }
        _ => String::new(),
    }
}

fn match_msg(err: ParserError) -> String {
    match err {
    ParserError::ExpectedFound(&TokenType::LeftBrace, got, position) => format!("When parsing a match expression on line {}, column {}, we were looking for a left brace {}, but we found something else ({}).

 This most likely means that you accidentally used the match operator (::), or you just forgot a left bracket when opening the match", position.line, position.column, bold(&yellow("{")), got),
    _ => String::new()
  }
}

pub fn generate_parser_message(
    err: ParserError,
    t: ParserType,
    pos: Position,
    source: &str,
) -> String {
    let mut msg = match t {
        ParserType::Assign => assign_msg(err),
        ParserType::Array => array_msg(err),
        ParserType::Match => match_msg(err),
        _ => String::new(),
    };

    msg.push('\n');

    let error = generate_pretty_error(pos, source);

    msg.push_str(&error);

    msg
}

pub fn generate_pretty_error(pos: Position, source: &str) -> String {
    let lines: Vec<&str> = source.split('\n').collect();
    let line = lines[pos.line - 1];

    let mut message = format!("{}. ", pos.line);

    let offset = message.len();

    message.push_str(line);

    message.push('\n');
    for _ in 1..pos.column + offset {
        message.push(' ');
    }
    message.push_str("^-- Here");

    message
}
