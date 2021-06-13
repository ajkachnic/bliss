use std::{fmt, ops::Range};

use crate::{
    context::{Context, Hint},
    location::{Location, PreciseLocation},
    style::{red, yellow},
    token::TokenType,
};
use thiserror::Error;

pub type Position = Range<usize>;

// use crate::style::{bold, yellow};

pub type ParseResult<T> = Result<T, ParseError>;
// All the different types of parsers (like parse_array, parse_match, etc)
#[derive(Debug, Clone)]
pub enum ParseType {
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
    Pattern,
}

#[derive(Debug, PartialEq, Error, Clone)]
pub enum ParseErrorKind {
    #[error("Expected token {expected:?}, found {found:?} instead")]
    ExpectedFound {
        expected: TokenType,
        found: TokenType,
    },
    #[error("Expected one of {expected:?}, found {found:?} instead")]
    ExpectedMultiple {
        expected: Vec<TokenType>,
        found: TokenType,
    },
    #[error("No prefix parser found for token {0:?}")]
    NoPrefixFound(TokenType),
    #[error("Token {0:?} is not supported in this structure")]
    UnsupportedToken(TokenType),
}

#[derive(Debug, Error, Clone)]
pub struct ParseError {
    #[source]
    pub kind: ParseErrorKind,
    pub position: Position,
    context: Vec<String>,
    hint: Option<String>,
    source: String,
}

impl Context for ParseError {
    fn context<T: ToString>(&self, context: T) -> Self {
        let mut clone = self.clone();
        clone.context.push(context.to_string());
        clone
    }

    fn get_context(&self) -> Vec<String> {
        self.context.clone()
    }
}

impl<T: Clone> Context for Result<T, ParseError> {
    fn context<A: ToString>(&self, context: A) -> Self {
        self.clone().map_err(|e| e.context(context))
    }

    fn get_context(&self) -> Vec<String> {
        match self {
            Ok(_) => vec![],
            Err(err) => err.get_context(),
        }
    }
}

impl Hint for ParseError {
    fn hint<T: ToString>(&self, hint: T) -> Self {
        let mut clone = self.clone();
        clone.hint = Some(hint.to_string());
        clone
    }
}

impl<T: Clone> Hint for Result<T, ParseError> {
    fn hint<A: ToString>(&self, hint: A) -> Self {
        self.clone().map_err(|e| e.hint(hint))
    }
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, position: Position, source: String) -> Self {
        ParseError {
            kind,
            position,
            source,
            context: vec![],
            hint: None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hint = match self.hint.clone() {
            Some(hint) => format!("\n{}: {}", yellow("Hint"), hint),
            None => String::new(),
        };

        let precise = PreciseLocation::new(
            Location::from(self.position.start, self.source.as_str()),
            Location::from(self.position.start, self.source.as_str()),
        );

        let lines: Vec<&str> = self.source.lines().collect();
        let relevant = if precise.0.start.line == precise.0.end.line {
            vec![*(lines.get(precise.0.start.line - 1).unwrap())]
        } else {
            Vec::from(
                lines
                    .get(precise.0.start.line - 1..=precise.0.end.line - 1)
                    .unwrap(),
            )
        };
        let mut current_line = precise.0.start.line;
        for line in relevant.iter() {
            writeln!(f, "{}|  {}", current_line, line)?;
            current_line += 1;
        }

        let offset = ((current_line - 1).to_string().len() + 3) + (precise.0.start.column - 1);

        let bump = String::from(" ").repeat(offset);
        writeln!(f, "{}{}", bump, red("^"))?;
        writeln!(f, "{}", self.kind)?;

        writeln!(f, "{}\n{}", hint, self.display_context())
    }
}

// fn assign_msg(err: ParserError) -> String {
//     match err {
//     ParserError::ExpectedFound(&TokenType::Assign, got, position) => {
//       return format!("When parsing an assignment statement on line {}, column {}, we were looking a {}, but we found something else ({:?}). There's a good change you forgot to put an equals sign, or put another token before it.", position.line, position.column, bold(&yellow("=")), got)
//     },
//     // The wildcard is safe here because this is the only expect_peek
//     _ => String::new()
//   }
// }

// fn array_msg(err: ParserError) -> String {
//     match err {
//         ParserError::ExpectedFound(&TokenType::RightBracket, got, position) => {
//             let got = format!("{}", got);
//             format!("When parsing an array on line {}, column {}, we were looking for right bracket {}, but we found something else ({}).

// Hint: Double check to make sure you've closed all your arrays, and you should be on your way", position.line, position.column,emphasize("]"), emphasize(&got))
//         }
//         _ => String::new(),
//     }
// }

// fn match_msg(err: ParserError) -> String {
//     match err {
//     ParserError::ExpectedFound(&TokenType::LeftBrace, got, position) => format!("When parsing a match expression on line {}, column {}, we were looking for a left brace {}, but we found something else ({}).

//  This most likely means that you accidentally used the match operator (::), or you just forgot a left bracket when opening the match", position.line, position.column, bold(&yellow("{")), got),
//     _ => String::new()
//   }
// }
