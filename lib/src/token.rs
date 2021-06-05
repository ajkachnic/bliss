use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Illegal,
    EOF,

    Ident(String),  // foobar
    Number(f64),    // Integer or float
    String(String), // "hello world"
    Symbol(String), // Self representing value, like :true

    // Symbols and Operators
    Assign,   // =
    Plus,     // +
    Minus,    // -
    Asterisk, // *
    Modulus,  // %
    Slash,    // /
    Period,   // .
    Arrow,    // ->
    Range,    // .. (like 0..5)
    Match,    // ::
    // Boolean operators
    Greater,   // >
    Less,      // <
    GreaterEq, // >=
    LessEq,    // <=
    Eq,        // ==
    NotEq,     // !=
    Bang,      // !

    // Logical operators
    And, // &&
    Or,  // ||

    // Delimiters
    Comma,
    Semicolon,
    Colon, // :

    // Braces 'n stuff
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,

    // Keywords
    // Might be better to split off into a second enum

    // Import related
    Import,
    From,
    As,

    Return,
    Function,
    True,
    False,
    If,
    Else,
    Then,
    Let,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Ident(value) => write!(f, "{}", value),
            Token::Number(value) => write!(f, "{}", value),
            Token::String(value) => write!(f, "'{}'", value),
            Token::Symbol(value) => write!(f, ":{}", value),

            Token::Assign => write!(f, "="),   // =
            Token::Plus => write!(f, "+"),     // +
            Token::Minus => write!(f, "-"),    // -
            Token::Asterisk => write!(f, "*"), // *
            Token::Modulus => write!(f, "%"),  // %
            Token::Slash => write!(f, "/"),    // /
            Token::Period => write!(f, "."),   // .
            Token::Arrow => write!(f, "->"),   // ->
            Token::Range => write!(f, ".."),   // .. (like 0..5)
            Token::Match => write!(f, "::"),
            Token::Greater => write!(f, ">"),    // >
            Token::Less => write!(f, "<"),       // <
            Token::GreaterEq => write!(f, ">="), // >=
            Token::LessEq => write!(f, "<="),    // <=
            Token::Eq => write!(f, "=="),        // ==
            Token::NotEq => write!(f, "!="),     // !=
            Token::Bang => write!(f, "!"),       // !

            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),

            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"), // :

            // Braces 'n stuff
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),

            // Keywords
            // Might be better to split off into a second enum

            // Import related
            Token::Import => write!(f, "import"),
            Token::From => write!(f, "from"),
            Token::As => write!(f, "as"),

            Token::Return => write!(f, "return"),
            Token::Function => write!(f, "fn"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Then => write!(f, "then"),

            Token::EOF => write!(f, "EOF"),
            _ => write!(f, ""),
        }
    }
}

pub fn lookup_keyword(name: &str) -> Token {
    match name {
        "import" => Token::Import,
        "from" => Token::From,
        "as" => Token::As,
        "fn" => Token::Function,
        "return" => Token::Return,
        "true" => Token::True,
        "false" => Token::False,
        "if" => Token::If,
        "then" => Token::Then,
        "else" => Token::Else,
        "let" => Token::Let,
        _ => Token::Ident(name.to_string()),
    }
}
