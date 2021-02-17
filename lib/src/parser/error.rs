use crate::{style::emphasize, token::Token};

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
    ExpectedFound(&'a Token, &'a Token),
    // NoPrefixFound(&'a Token),
}

fn assign_msg(err: ParserError) -> String {
    match err {
    ParserError::ExpectedFound(&Token::Assign, got) => {
      return format!("When parsing an assignment statement, we were looking a {}, but we found something else ({:?}). There's a good change you forgot to put an equals sign, or put another token before it.", bold(&yellow("=")), got)
    },
    // The wildcard is safe here because this is the only expect_peek
    _ => String::new()
  }
}

fn array_msg(err: ParserError) -> String {
    match err {
        ParserError::ExpectedFound(&Token::RightBracket, got) => {
            let got = format!("{}", got);
            format!("When parsing an array, we were looking for right bracket {}, but we found something else ({}).
    
Hint: Double check to make sure you've closed all your arrays, and you should be on your way", emphasize("]"), emphasize(&got))
        }
        _ => String::new(),
    }
}

fn match_msg(err: ParserError) -> String {
    match err {
    ParserError::ExpectedFound(&Token::LeftBrace, got) => format!("When parsing a match expression, we were looking for a left brace {}, but we found something else ({}).

 This most likely means that you accidentally used the match operator (::), or you just forgot a left bracket when opening the match", bold(&yellow("{")), got),
    _ => String::new()
  }
}

pub fn generate_parser_message(err: ParserError, t: ParserType) -> String {
    match t {
        ParserType::Assign => assign_msg(err),
        ParserType::Array => array_msg(err),
        ParserType::Match => match_msg(err),
        _ => String::new(),
    }
}
