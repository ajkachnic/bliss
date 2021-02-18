use super::*;
use crate::token::TokenType;
// Currently preferring many smaller tests over one giant one for a couple of reasons
// - More readable
// - Each test has an actual purpose
// - It's easier to tell where a bug may be
fn test_tokens(input: &str, tests: Vec<TokenType>) {
    let mut l = Lexer::new(input);
    for test in tests {
        let tok = l.next_token();
        println!("{:?}", tok);
        assert_eq!(tok.tok, test);
    }
}

#[test]
fn test_symbols() {
    let input = ":true + :false";
    let tests = vec![
        TokenType::Symbol(String::from("true")),
        TokenType::Plus,
        TokenType::Symbol(String::from("false")),
    ];
    test_tokens(input, tests);
}

#[test]
fn test_two_chars() {
    let input = "a != b == c >= d <= e :: 0..5 ->";
    let tests = vec![
        TokenType::Ident(String::from("a")),
        TokenType::NotEq,
        TokenType::Ident(String::from("b")),
        TokenType::Eq,
        TokenType::Ident(String::from("c")),
        TokenType::GreaterEq,
        TokenType::Ident(String::from("d")),
        TokenType::LessEq,
        TokenType::Ident(String::from("e")),
        TokenType::Match,
        TokenType::Number(String::from("0")),
        TokenType::Range,
        TokenType::Number(String::from("5")),
        TokenType::Arrow,
    ];
    test_tokens(input, tests);
}

#[test]
fn test_identifiers() {
    let input = "abc my_number5 foo bar foobar";
    let tests = vec![
        TokenType::Ident(String::from("abc")),
        TokenType::Ident(String::from("my_number5")),
        TokenType::Ident(String::from("foo")),
        TokenType::Ident(String::from("bar")),
        TokenType::Ident(String::from("foobar")),
    ];
    test_tokens(input, tests);
}

#[test]
fn test_strings() {
    let input = "'hello ' + \"world\"";
    let tests = vec![
        TokenType::String(String::from("hello ")),
        TokenType::Plus,
        TokenType::String(String::from("world")),
    ];
    test_tokens(input, tests);
}

#[test]
fn test_keywords() {
    let input = "import stuff from 'the place'";
    let tests = vec![
        TokenType::Import,
        TokenType::Ident(String::from("stuff")),
        TokenType::From,
        TokenType::String(String::from("the place")),
    ];
    test_tokens(input, tests);
}

#[test]
fn test_numbers() {
    let input = "5 + 4 * 8000";
    let tests = vec![
        TokenType::Number(String::from("5")),
        TokenType::Plus,
        TokenType::Number(String::from("4")),
        TokenType::Asterisk,
        TokenType::Number(String::from("8000")),
    ];
    test_tokens(input, tests);
}

#[test]
fn test_single() {
    let input = "=+-*/%(){},;:";
    let tests = vec![
        TokenType::Assign,
        TokenType::Plus,
        TokenType::Minus,
        TokenType::Asterisk,
        TokenType::Slash,
        TokenType::Modulus,
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::Comma,
        TokenType::Semicolon,
        TokenType::Colon,
        TokenType::EOF,
    ];
    test_tokens(input, tests);
}
