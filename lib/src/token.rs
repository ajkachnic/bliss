use std::fmt;

#[derive(Debug, Clone)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn from(offset: usize, source: &str) -> Position {
        let lines = source.split('\n');
        let mut line = 0;
        let mut column = 0;

        let mut current = 0;

        for (index, current_line) in lines.enumerate() {
            // Account for the newline
            current += current_line.len() + 1;
            // It's in this line
            if offset < current {
                line = index + 1;
                // I honestly don't even know why this needs the minus 2, but it works so i'm not complaining
                column = current - offset + 2;
                break;
            }
        }

        Position { line, column }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Token {
    pub tok: TokenType,
    pub offset: usize,
}

impl Default for Token {
    fn default() -> Self {
        Self::new()
    }
}

impl Token {
    pub fn new() -> Token {
        Token {
            tok: TokenType::EOF,
            offset: 0,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum TokenType {
    Illegal,
    EOF,

    Ident(String),  // foobar
    Number(String), // Integer or float
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
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Ident(value) => write!(f, "{}", value),
            TokenType::Number(value) => write!(f, "{}", value),
            TokenType::String(value) => write!(f, "'{}'", value),
            TokenType::Symbol(value) => write!(f, ":{}", value),

            TokenType::Assign => write!(f, "="),   // =
            TokenType::Plus => write!(f, "+"),     // +
            TokenType::Minus => write!(f, "-"),    // -
            TokenType::Asterisk => write!(f, "*"), // *
            TokenType::Modulus => write!(f, "%"),  // %
            TokenType::Slash => write!(f, "/"),    // /
            TokenType::Period => write!(f, "."),   // .
            TokenType::Arrow => write!(f, "->"),   // ->
            TokenType::Range => write!(f, ".."),   // .. (like 0..5)
            TokenType::Match => write!(f, "::"),
            TokenType::Greater => write!(f, ">"),    // >
            TokenType::Less => write!(f, "<"),       // <
            TokenType::GreaterEq => write!(f, ">="), // >=
            TokenType::LessEq => write!(f, "<="),    // <=
            TokenType::Eq => write!(f, "=="),        // ==
            TokenType::NotEq => write!(f, "!="),     // !=
            TokenType::Bang => write!(f, "!"),       // !

            TokenType::And => write!(f, "&&"),
            TokenType::Or => write!(f, "||"),

            TokenType::Comma => write!(f, ","),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Colon => write!(f, ":"), // :

            // Braces 'n stuff
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),

            // Keywords
            // Might be better to split off into a second enum

            // Import related
            TokenType::Import => write!(f, "import"),
            TokenType::From => write!(f, "from"),
            TokenType::As => write!(f, "as"),

            TokenType::Return => write!(f, "return"),
            TokenType::Function => write!(f, "fn"),
            TokenType::True => write!(f, "true"),
            TokenType::False => write!(f, "false"),
            TokenType::If => write!(f, "if"),
            TokenType::Else => write!(f, "else"),
            TokenType::Then => write!(f, "then"),

            TokenType::EOF => write!(f, "EOF"),
            _ => write!(f, ""),
        }
    }
}

pub fn lookup_keyword(name: &str) -> TokenType {
    match name {
        "import" => TokenType::Import,
        "from" => TokenType::From,
        "as" => TokenType::As,
        "fn" => TokenType::Function,
        "return" => TokenType::Return,
        "true" => TokenType::True,
        "false" => TokenType::False,
        "if" => TokenType::If,
        "then" => TokenType::Then,
        "else" => TokenType::Else,
        _ => TokenType::Ident(name.to_string()),
    }
}
