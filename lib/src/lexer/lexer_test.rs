use super::*;
use crate::token::Token;
// Currently preferring many smaller tests over one giant one for a couple of reasons
// - More readable
// - Each test has an actual purpose
// - It's easier to tell where a bug may be
fn test_tokens(input: &str, tests: Vec<Token>) {
    let mut l = Lexer::new(input);
    for test in tests {
        let tok = l.next_token();
        println!("{:?}", tok);
        assert_eq!(tok, test);
    }
}

#[test]
fn test_symbols() {
    let input = ":true + :false";
    let tests = vec![
        Token::Symbol(String::from("true")),
        Token::Plus,
        Token::Symbol(String::from("false")),
    ];
    test_tokens(input, tests);
}

#[test]
fn test_two_chars() {
    let input = "a != b == c >= d <= e :: 0..5 ->";
    let tests = vec![
        Token::Ident(String::from("a")),
        Token::NotEq,
        Token::Ident(String::from("b")),
        Token::Eq,
        Token::Ident(String::from("c")),
        Token::GreaterEq,
        Token::Ident(String::from("d")),
        Token::LessEq,
        Token::Ident(String::from("e")),
        Token::Match,
        Token::Number(String::from("0")),
        Token::Range,
        Token::Number(String::from("5")),
        Token::Arrow,
    ];
    test_tokens(input, tests);
}

#[test]
fn test_identifiers() {
    let input = "abc my_number5 foo bar foobar";
    let tests = vec![
        Token::Ident(String::from("abc")),
        Token::Ident(String::from("my_number5")),
        Token::Ident(String::from("foo")),
        Token::Ident(String::from("bar")),
        Token::Ident(String::from("foobar")),
    ];
    test_tokens(input, tests);
}

#[test]
fn test_strings() {
    let input = "'hello ' + \"world\"";
    let tests = vec![
        Token::String(String::from("hello ")),
        Token::Plus,
        Token::String(String::from("world")),
    ];
    test_tokens(input, tests);
}

#[test]
fn test_keywords() {
    let input = "import stuff from 'the place'";
    let tests = vec![
        Token::Import,
        Token::Ident(String::from("stuff")),
        Token::From,
        Token::String(String::from("the place")),
    ];
    test_tokens(input, tests);
}

#[test]
fn test_numbers() {
    let input = "5 + 4 * 8000";
    let tests = vec![
        Token::Number(String::from("5")),
        Token::Plus,
        Token::Number(String::from("4")),
        Token::Asterisk,
        Token::Number(String::from("8000")),
    ];
    test_tokens(input, tests);
}

#[test]
fn test_single() {
    let input = "=+-*/%(){},;:";
    let tests = vec![
        Token::Assign,
        Token::Plus,
        Token::Minus,
        Token::Asterisk,
        Token::Slash,
        Token::Modulus,
        Token::LeftParen,
        Token::RightParen,
        Token::LeftBrace,
        Token::RightBrace,
        Token::Comma,
        Token::Semicolon,
        Token::Colon,
        Token::EOF,
    ];
    test_tokens(input, tests);
}
