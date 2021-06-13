use crate::ast::{BlockStatement, Expr, Ident, Pattern, Program, Stmt};
use crate::context::{Context, Hint};
use crate::lexer::Lexer;
use crate::location::Position;
use crate::token::{Token, TokenType};

use error::{ParseError, ParseErrorKind, ParseResult};

pub mod error;

#[cfg(test)]
#[path = "./parser_test.rs"]
mod parser_test;

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Logical,
    Equals,
    LessGreater,
    Range,
    Sum,
    Product,
    Modulus,
    Prefix,
    Match,
    Call,
}

fn get_precedence(tok: &TokenType) -> Precedence {
    match tok {
        TokenType::And | TokenType::Or => Precedence::Logical,
        TokenType::Eq => Precedence::Equals,
        TokenType::NotEq => Precedence::Equals,

        TokenType::Less => Precedence::LessGreater,
        TokenType::Greater => Precedence::LessGreater,
        TokenType::LessEq => Precedence::LessGreater,
        TokenType::GreaterEq => Precedence::LessGreater,

        TokenType::Range => Precedence::Range,

        TokenType::Plus => Precedence::Sum,
        TokenType::Minus => Precedence::Sum,

        TokenType::Slash => Precedence::Product,
        TokenType::Asterisk => Precedence::Product,

        TokenType::Modulus => Precedence::Modulus,

        TokenType::Match => Precedence::Match,

        TokenType::LeftParen => Precedence::Call,

        _ => Precedence::Lowest,
    }
}

pub struct Parser<'a> {
    l: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
    source: String,
}

impl<'a> Parser<'a> {
    pub fn new(l: Lexer, source: String) -> Parser {
        let mut p = Parser {
            l,
            current_token: Token::new(),
            peek_token: Token::new(),
            source,
        };

        p.next_token();
        p.next_token();
        p
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.l.next_token();
    }

    fn position(&self) -> Position {
        self.current_token.position.clone()
    }

    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let mut stmts = Program::new();
        while self.current_token.tok != TokenType::Eof {
            let stmt = self.parse_stmt().context("Parsing program")?;
            stmts.0.push(stmt);
            self.next_token();
        }
        Ok(stmts)
    }

    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        let pattern = match self.current_token.clone().tok {
            TokenType::Ident(id) => {
                if id.as_str() == "_" {
                    Pattern::Nothing
                } else {
                    Pattern::Ident(Ident(id))
                }
            }
            TokenType::Symbol(sym) => Pattern::Symbol(sym),
            TokenType::String(str) => Pattern::String(str),
            TokenType::True => Pattern::Boolean(true),
            TokenType::False => Pattern::Boolean(false),
            TokenType::Number(num) => Pattern::Number(num),
            TokenType::LeftBracket => {
                let mut items = vec![];
                if self.peek_token_is(&TokenType::RightBracket) {
                    self.next_token();
                    self.next_token();
                    Pattern::Array(items)
                } else {
                    self.next_token();
                    let value = self
                        .parse_pattern()
                        .context("Parsing array pattern")
                        .context("Parsing pattern")?;
                    items.push(value);

                    while self.peek_token_is(&TokenType::Comma) {
                        self.next_token();
                        self.next_token();
                        let value = self
                            .parse_pattern()
                            .context("Parsing array pattern")
                            .context("Parsing pattern")?;
                        items.push(value);
                    }

                    // Read past the RightBracket
                    self.expect_peek(&TokenType::RightBracket)
                        .context("Parsing closing bracket")
                        .context("Parsing array pattern")
                        .context("Parsing pattern")
                        .hint(
                            "Double check to make sure you're closing all your closing brackets",
                        )?;

                    Pattern::Array(items)
                }
            }
            TokenType::LeftBrace => {
                let mut items = vec![];
                if self.peek_token_is(&TokenType::RightBrace) {
                    self.next_token();
                    self.next_token();
                    Pattern::Hash(items)
                } else {
                    self.next_token();
                    let value = self
                        .parse_identifier()
                        .context("Parsing hash identifier")
                        .context("Parsing hash pattern")
                        .context("Parsing pattern")?;
                    items.push((value, None));

                    while self.peek_token_is(&TokenType::Comma) {
                        self.next_token();
                        self.next_token();
                        let value = self
                            .parse_identifier()
                            .context("Parsing hash identifier")
                            .context("Parsing hash pattern")
                            .context("Parsing pattern")?;
                        items.push((value, None));
                    }

                    // Read past the RightBracket
                    self.expect_peek(&TokenType::RightBrace)
                        .context("Parsing closing brace")
                        .context("Parsing hash pattern")
                        .context("Parsing pattern")
                        .hint("Double check to make sure you're closing all your closing braces")?;

                    Pattern::Hash(items)
                }
            }
            tok => {
                return Err(ParseError::new(
                    ParseErrorKind::UnsupportedToken(tok),
                    self.position(),
                    self.source.clone(),
                ))
            }
        };

        Ok(pattern)
    }

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.current_token.clone().tok {
            TokenType::Let => self.parse_assign_stmt(),
            TokenType::Return => self.parse_return_stmt(),
            TokenType::Import => self.parse_import_stmt(),
            _ => self.parse_expression_stmt(),
        }
    }

    fn parse_assign_stmt(&mut self) -> ParseResult<Stmt> {
        self.next_token();
        let name = self
            .parse_pattern()
            .context("Parsing assignment name")
            .context("Parsing assignment")?;
        // Should be an equals sign
        self.expect_peek(&TokenType::Assign)
            .context("Parsing assignment")?;

        self.next_token();
        let value = self
            .parse_expression(Precedence::Lowest)
            .context("Parsing assignment value")?;
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        Ok(Stmt::Assign(name, value))
    }
    fn parse_return_stmt(&mut self) -> ParseResult<Stmt> {
        self.next_token();
        let value = self
            .parse_expression(Precedence::Lowest)
            .context("Parsing assignment value")?;
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        Ok(Stmt::Return(value))
    }
    fn parse_import_stmt(&mut self) -> ParseResult<Stmt> {
        self.next_token();
        let name = self
            .parse_pattern()
            .context("Parsing import path/name")
            .context("Parsing import")?;
        self.expect_peek(&TokenType::From)
            .context("Parsing import statement")?;
        self.next_token();
        let source = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        Ok(Stmt::Import { source, name })
    }
    fn parse_expression_stmt(&mut self) -> ParseResult<Stmt> {
        let expression = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token()
        }
        Ok(Stmt::Expr(expression))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> ParseResult<Expr> {
        let token = self.current_token.clone();
        let mut left = match token.tok {
            TokenType::Ident(_) => self.parse_identifier().map(Expr::Ident),
            TokenType::String(_) => self.parse_string(),
            TokenType::Symbol(_) => self.parse_symbol(),
            TokenType::Number(_) => self.parse_number(),
            TokenType::Bang => self.parse_prefix_expression(),
            TokenType::Minus => self.parse_prefix_expression(),
            TokenType::True | TokenType::False => Ok(self.parse_boolean()),
            TokenType::LeftParen => self.parse_grouped_expressions(),
            TokenType::If => self.parse_if_expression(),
            TokenType::Function => self.parse_function(),
            TokenType::LeftBracket => self.parse_array(),
            TokenType::LeftBrace => self.parse_hash(),
            _ => {
                return Err(self.no_prefix_parser_error(token));
            }
        };
        while !self.peek_token_is(&TokenType::Semicolon) && precedence < self.peek_precedence() {
            left = match self.peek_token.tok {
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Range
                | TokenType::Asterisk
                | TokenType::Slash
                | TokenType::Modulus
                | TokenType::Eq
                | TokenType::NotEq
                | TokenType::Greater
                | TokenType::GreaterEq
                | TokenType::And
                | TokenType::Or
                | TokenType::Less
                | TokenType::LessEq => {
                    self.next_token();
                    self.parse_infix_expression(left?)
                }
                TokenType::LeftParen => {
                    self.next_token();
                    self.parse_call_expression(left?)
                }
                TokenType::Match => {
                    self.next_token();
                    self.parse_match(left?)
                }
                _ => return left,
            };
        }
        left
    }

    fn parse_identifier(&mut self) -> ParseResult<Ident> {
        if let TokenType::Ident(ident) = self.current_token.clone().tok {
            return Ok(Ident(ident));
        }
        unreachable!()
    }
    fn parse_number(&mut self) -> ParseResult<Expr> {
        if let TokenType::Number(num) = self.current_token.clone().tok {
            return Ok(Expr::Number(num));
        }
        unreachable!()
    }
    fn parse_boolean(&mut self) -> Expr {
        let value = self.current_token_is(&TokenType::True);
        Expr::Boolean(value)
    }
    fn parse_string(&mut self) -> ParseResult<Expr> {
        if let TokenType::String(value) = self.current_token.clone().tok {
            return Ok(Expr::String(value));
        }
        unreachable!()
    }
    fn parse_symbol(&mut self) -> ParseResult<Expr> {
        if let TokenType::Symbol(value) = self.current_token.clone().tok {
            return Ok(Expr::Symbol(value));
        }
        unreachable!()
    }
    fn parse_prefix_expression(&mut self) -> ParseResult<Expr> {
        let operator = match self.current_token.tok {
            TokenType::Bang => "!",
            TokenType::Minus => "-",
            _ => "",
        };
        self.next_token();
        let right = self
            .parse_expression(Precedence::Prefix)
            .context("Parsing prefix expression")?;
        Ok(Expr::Prefix(operator.to_string(), Box::new(right)))
    }
    fn parse_infix_expression(&mut self, left: Expr) -> ParseResult<Expr> {
        let operator = match self.current_token.tok {
            TokenType::Plus => "+",
            TokenType::Minus => "-",
            TokenType::Asterisk => "*",
            TokenType::Slash => "/",
            TokenType::Modulus => "%",
            TokenType::Eq => "==",
            TokenType::NotEq => "!=",
            TokenType::Greater => ">",
            TokenType::GreaterEq => ">=",
            TokenType::Less => "<",
            TokenType::LessEq => "<=",
            TokenType::Range => "..",
            TokenType::And => "&&",
            TokenType::Or => "||",
            _ => "",
        };

        let precedence = self.current_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;

        Ok(Expr::Infix(
            Box::new(left),
            operator.to_string(),
            Box::new(right),
        ))
    }
    fn parse_grouped_expressions(&mut self) -> ParseResult<Expr> {
        self.next_token();
        let exp = self
            .parse_expression(Precedence::Lowest)
            .context("Parsing grouped expression")?;
        self.expect_peek(&TokenType::RightParen)
            .context("Parsing closing paren")
            .context("Parsing grouped expression")?;
        Ok(exp)
    }
    fn parse_if_expression(&mut self) -> ParseResult<Expr> {
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;
        // TODO: Support shortened versions like this:
        //  if true then 5 else 10
        self.expect_peek(&TokenType::LeftBrace)?;
        let consequence = self.parse_block_stmt()?;
        self.expect_peek(&TokenType::Else)?;
        self.expect_peek(&TokenType::LeftBrace)?;
        let alternative = self.parse_block_stmt()?;

        Ok(Expr::If {
            condition: Box::new(condition),
            consequence,
            alternative,
        })
    }
    fn parse_function(&mut self) -> ParseResult<Expr> {
        let token = self.peek_token.clone();
        let parameters = match token.tok {
            TokenType::LeftParen => {
                self.next_token();
                self.parse_function_parameters()?
            }
            TokenType::Ident(ident) => {
                self.next_token();
                vec![Ident(ident)]
            }
            tok => {
                return Err(ParseError::new(
                    ParseErrorKind::ExpectedMultiple {
                        expected: vec![TokenType::LeftParen, TokenType::Ident(String::new())],
                        found: tok,
                    },
                    self.position(),
                    self.source.clone(),
                )
                .context("Parsing function"))
            }
        };

        self.expect_peek(&TokenType::Arrow)
            .context("Expecting arrow")
            .context("Parsing function")?;
        self.next_token();

        let body = self.parse_block_shorthand()?;

        Ok(Expr::Function { parameters, body })
    }
    fn parse_function_parameters(&mut self) -> ParseResult<Vec<Ident>> {
        let mut identifiers = vec![];

        if self.peek_token_is(&TokenType::RightParen) {
            self.next_token();
            return Ok(identifiers);
        }

        self.next_token();

        let ident = match &self.current_token.tok {
            TokenType::Ident(name) => Ident(name.clone()),
            _ => Ident(String::new()),
        };
        identifiers.push(ident);

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            let ident = match &self.current_token.tok {
                TokenType::Ident(name) => Ident(name.clone()),
                _ => Ident(String::new()),
            };
            identifiers.push(ident);
        }

        self.expect_peek(&TokenType::RightParen)
            .context("Parsing closing paren")
            .context("Parsing function parameters")?;

        Ok(identifiers)
    }
    fn parse_call_expression(&mut self, function: Expr) -> ParseResult<Expr> {
        let args = self
            .parse_call_expression_args()
            .context("Parsing call expression")?;
        Ok(Expr::Call {
            function: Box::new(function),
            arguments: args,
        })
    }
    fn parse_call_expression_args(&mut self) -> ParseResult<Vec<Expr>> {
        let mut args = vec![];

        if self.peek_token_is(&TokenType::RightParen) {
            self.next_token();
            return Ok(args);
        }

        self.next_token();

        args.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            args.push(self.parse_expression(Precedence::Lowest)?);
        }

        self.expect_peek(&TokenType::RightParen)
            .context("Parsing closing paren")
            .context("Parsing call expression arguments")?;

        Ok(args)
    }

    fn parse_block_stmt(&mut self) -> ParseResult<BlockStatement> {
        let mut stmts = vec![];
        self.next_token();
        // self.next_token();
        while !self.current_token_is(&TokenType::RightBrace)
            && !self.current_token_is(&TokenType::Eof)
        {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
            self.next_token();
        }

        Ok(BlockStatement(stmts))
    }

    fn parse_block_shorthand(&mut self) -> ParseResult<BlockStatement> {
        if self.current_token_is(&TokenType::LeftBrace) {
            return self.parse_block_stmt();
        }
        let expr = self.parse_expression(Precedence::Lowest)?;
        Ok(Stmt::Expr(expr).into())
    }

    fn parse_match(&mut self, condition: Expr) -> ParseResult<Expr> {
        self.expect_peek(&TokenType::LeftBrace)?;
        let cases = self.parse_match_cases()?;
        self.expect_peek(&TokenType::RightBrace)
            .context("Parsing closing bracket")
            .context("Parsing match expression")
            .hint("Ensure you didn't forget a curly brace to close your match expression")?;
        Ok(Expr::Match {
            condition: Box::new(condition),
            cases,
        })
    }
    fn parse_match_cases(&mut self) -> ParseResult<Vec<(Pattern, BlockStatement)>> {
        let mut cases = vec![];

        if self.peek_token_is(&TokenType::RightBrace) {
            return Ok(cases);
        }
        self.next_token();
        let key = self.parse_pattern()?;
        self.expect_peek(&TokenType::Arrow)
            .context("Parsing match case")
            .hint(
                "Make sure you follow valid match syntax, here's an example:
true :: {
    true -> 'yes',
    false -> 'no'    
}
",
            )?;
        self.next_token();
        let value = self.parse_block_shorthand()?;

        cases.push((key, value));

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            let key = self.parse_pattern().context("Parsing match expression")?;
            self.expect_peek(&TokenType::Arrow)
                .context("Parsing match case")
                .hint(
                    "Make sure you follow valid match syntax, here's an example:
true :: {
    true -> 'yes',
    false -> 'no'    
}
",
                )?;
            self.next_token();
            let value = self.parse_block_shorthand()?;

            cases.push((key, value));
        }
        Ok(cases)
    }

    fn parse_array(&mut self) -> ParseResult<Expr> {
        let mut items = vec![];
        if self.peek_token_is(&TokenType::RightBracket) {
            self.next_token();
            return Ok(Expr::Array(items));
        }

        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;
        items.push(value);

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            let value = self.parse_expression(Precedence::Lowest)?;
            items.push(value);
        }

        // Read past the RightBracket
        self
            .expect_peek(&TokenType::RightBracket)
            .context("Looking for closing array token")
            .hint("Double check to make sure you've closed all your arrays, and you should be on your way")?;

        Ok(Expr::Array(items))
    }
    fn parse_hash(&mut self) -> ParseResult<Expr> {
        let mut items = vec![];
        if self.peek_token_is(&TokenType::RightBrace) {
            self.next_token();
            return Ok(Expr::Hash(items));
        }

        self.next_token();
        let key = self.parse_expression(Precedence::Lowest)?;
        match key.clone() {
            Expr::Ident(ident) => {
                // Short hand like this
                // { foo, bar = 5 }
                // The value of the foo key is the value of the variable foo
                if self.peek_token_is(&TokenType::Comma)
                    || self.peek_token_is(&TokenType::RightBrace)
                {
                    items.push((ident, key))
                } else {
                    self.expect_peek(&TokenType::Assign)
                        .context("Parsing hash value")
                        .context("Parsing hash")?;
                    self.next_token();
                    let value = self.parse_expression(Precedence::Lowest)?;
                    items.push((ident, value));
                }
            }
            _ => return Ok(Expr::Hash(items)),
        }

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            let key = self.parse_expression(Precedence::Lowest)?;
            match key.clone() {
                Expr::Ident(ident) => {
                    if self.peek_token_is(&TokenType::Comma) {
                        items.push((ident, key))
                    } else {
                        self.expect_peek(&TokenType::Assign)
                            .context("Parsing hash value")
                            .context("Parsing hash")?;
                        self.next_token();
                        let value = self.parse_expression(Precedence::Lowest)?;
                        items.push((ident, value));
                    }
                }
                _ => return Ok(Expr::Hash(items)),
            }
        }

        // Read past the RightBracket
        self.expect_peek(&TokenType::RightBrace)
            .context("Parsing closing bracket")
            .context("Parsing hash")?;

        Ok(Expr::Hash(items))
    }

    // Utils
    fn current_token_is(&mut self, t: &TokenType) -> bool {
        &self.current_token.tok == t
    }
    fn peek_token_is(&mut self, t: &TokenType) -> bool {
        &self.peek_token.tok == t
    }
    fn expect_peek(&mut self, t: &TokenType) -> ParseResult<()> {
        if self.peek_token_is(t) {
            self.next_token();
            return Ok(());
        }
        Err(self.peek_error(&t))
    }
    fn peek_precedence(&mut self) -> Precedence {
        get_precedence(&self.peek_token.tok)
    }
    fn current_precedence(&mut self) -> Precedence {
        get_precedence(&self.current_token.tok)
    }
    // Errors stuff
    fn peek_error(&mut self, t: &TokenType) -> ParseError {
        println!("{:?}", self.peek_token);
        let t = Token {
            tok: t.clone(),
            position: match self.peek_token.tok {
                TokenType::Eof => self.current_token.position.clone(),
                _ => self.peek_token.position.clone(),
            },
        };

        let kind = ParseErrorKind::ExpectedFound {
            expected: t.tok,
            found: self.peek_token.tok.clone(),
        };
        ParseError::new(kind, t.position, self.source.clone())
    }
    fn no_prefix_parser_error(&mut self, t: Token) -> ParseError {
        ParseError::new(
            ParseErrorKind::NoPrefixFound(t.tok),
            self.position(),
            self.source.clone(),
        )
    }
}
