use std::iter::Peekable;
use std::str::Chars;

use crate::token;
use token::{Token, TokenType};

#[cfg(test)]
#[path = "./lexer_test.rs"]
mod lexer_test;

type IsFunc = dyn Fn(char) -> bool;

// TODO: Add positions
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    offset: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &str) -> Lexer {
        Lexer {
            input: input.chars().peekable(),
            offset: 0,
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn multipeek(&mut self, dist: usize) -> Option<char> {
        let mut clone = self.input.clone();
        let mut ch = clone.next();
        for _ in 1..dist {
            ch = clone.next();
        }
        ch
    }

    fn read(&mut self) -> Option<char> {
        self.offset += 1;
        self.input.next()
    }

    fn peek_is(&mut self, expected: char) -> bool {
        let peek = self.peek();
        match peek {
            Some(&ch) => ch == expected,
            None => false,
        }
    }
    fn peek_not(&mut self, expected: char) -> bool {
        !self.peek_is(expected)
    }

    fn two_char(&mut self, expected: char, single: TokenType, double: TokenType) -> TokenType {
        if self.peek_is(expected) {
            self.read();
            return double;
        }
        single
    }

    fn generate_token(&mut self, ch: char) -> TokenType {
        match ch {
            // Symbols and Operators
            '=' => self.two_char('=', TokenType::Assign, TokenType::Eq),
            '+' => TokenType::Plus,
            '-' => self.two_char('>', TokenType::Minus, TokenType::Arrow),
            '*' => TokenType::Asterisk,
            '%' => TokenType::Modulus,
            '/' => TokenType::Slash,
            /*
            There are some complications with leading zero support though
            - It's potentially ambiguous (example ident.5)
            - We can't take things like this into context until parsing, so it might be better to handle this there (especially since that's also where we convert numbers to actual numbers)
             */
            '.' => self.two_char('.', TokenType::Period, TokenType::Range),
            // Equality Operators
            '>' => self.two_char('=', TokenType::Greater, TokenType::GreaterEq),
            '<' => self.two_char('=', TokenType::Less, TokenType::LessEq),
            '!' => self.two_char('=', TokenType::Bang, TokenType::NotEq),

            // Logical operators
            '|' => self.two_char('|', TokenType::Illegal, TokenType::Or),
            '&' => self.two_char('&', TokenType::Illegal, TokenType::And),

            // Delimiters
            ',' => TokenType::Comma,
            ';' => TokenType::Semicolon,
            ':' => {
                if let Some(&next) = self.peek() {
                    if Self::is_letter(next) {
                        self.read();
                        let text = self.read_identifier(next);
                        return TokenType::Symbol(text);
                    }
                }
                self.two_char(':', TokenType::Colon, TokenType::Match)
            }

            // Strings
            '\'' | '"' => {
                let string = self.read_string(ch);
                TokenType::String(string)
            }

            // Braces 'n stuff
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            '[' => TokenType::LeftBracket,
            ']' => TokenType::RightBracket,
            _ => {
                if Self::is_letter(ch) {
                    let ident = self.read_identifier(ch);
                    return token::lookup_keyword(ident.as_str());
                } else if Self::is_digit(ch) {
                    let number = self.read_number(ch);
                    match number {
                        Some(number) => return TokenType::Number(number),
                        None => return TokenType::Illegal,
                    }
                }
                TokenType::Illegal
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if let Some(ch) = self.read() {
            let start = self.offset;
            // Light abstraction to make this less ugly
            let tok = self.generate_token(ch);
            let end = self.offset;
            return Token {
                tok,
                position: start..end,
            };
        }
        Token {
            tok: TokenType::Eof,
            position: 0..0,
        }
    }

    // "is" functions
    fn is_whitespace(ch: char) -> bool {
        ch.is_ascii_whitespace()
    }
    fn is_letter(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }
    fn is_digit(ch: char) -> bool {
        ch.is_ascii_digit()
    }
    fn is_dot(ch: char) -> bool {
        ch == '.'
    }

    // Takes a function like &Self::is_letter
    fn peek_fn(&mut self, f: &IsFunc) -> bool {
        match self.peek() {
            Some(&ch) => f(ch),
            None => false,
        }
    }

    // Takes a function like &Self::is_letter
    fn multipeek_fn(&mut self, dist: usize, f: &IsFunc) -> bool {
        match self.multipeek(dist) {
            Some(ch) => f(ch),
            None => false,
        }
    }

    fn read_identifier(&mut self, initial: char) -> String {
        let mut ident = String::from(initial);
        // Allows letters and digits
        // This works because the initial character can only be a letter
        while self.peek_fn(&Self::is_letter) || self.peek_fn(&Self::is_digit) {
            if let Some(ch) = self.read() {
                ident.push(ch)
            }
        }
        ident
    }
    /*
      Float support added in parsing (it's really cursed)
      Because if there are tokens like this:
        Number(5), Period, Number(6)
      That unambiguously translates to Number(5.6) (i hope)
    */
    fn read_number(&mut self, initial: char) -> Option<f64> {
        let mut number = String::from(initial);
        let mut dot = false;
        while self.peek_fn(&Self::is_digit)
        // Prevent reading the range operator
            || (self.peek_fn(&Self::is_dot) && !self.multipeek_fn(2, &Self::is_dot))
        {
            if let Some(ch) = self.read() {
                if ch == '.' {
                    if !dot {
                        dot = true;
                    } else {
                        return None;
                    }
                }
                number.push(ch)
            }
        }
        match number.parse() {
            Ok(num) => Some(num),
            Err(_) => None,
        }
    }
    // TODO: Add support for escapes, like \"
    fn read_string(&mut self, initial: char) -> String {
        let mut string = String::new();
        // We use the initial character to support strings that use single or double quotes
        while self.peek_not(initial) {
            if let Some(ch) = self.read() {
                string.push(ch);
            } else {
                // TODO: Add proper error handling
                panic!("Reached end of input before string was terminated")
            }
        }
        // Read the last quote
        self.read();
        string
    }
    fn skip_whitespace(&mut self) {
        while self.peek_fn(&Self::is_whitespace) {
            self.read();
        }
    }
}
