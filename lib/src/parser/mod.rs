use ast::{BlockStatement, Expr, Ident, Pattern, Program, Stmt};
use error::{generate_parser_message, ParserError, ParserResult, ParserType};

use crate::ast;
use crate::lexer::Lexer;
use crate::token::Token;

mod error;

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

fn get_precedence(tok: &Token) -> Precedence {
    match tok {
        Token::And | Token::Or => Precedence::Logical,
        Token::Eq => Precedence::Equals,
        Token::NotEq => Precedence::Equals,

        Token::Less => Precedence::LessGreater,
        Token::Greater => Precedence::LessGreater,
        Token::LessEq => Precedence::LessGreater,
        Token::GreaterEq => Precedence::LessGreater,

        Token::Range => Precedence::Range,

        Token::Plus => Precedence::Sum,
        Token::Minus => Precedence::Sum,

        Token::Slash => Precedence::Product,
        Token::Asterisk => Precedence::Product,

        Token::Modulus => Precedence::Modulus,

        Token::Match => Precedence::Match,

        Token::LeftParen => Precedence::Call,

        _ => Precedence::Lowest,
    }
}

pub struct Parser<'a> {
    l: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(l: Lexer) -> Parser {
        let mut p = Parser {
            l,
            current_token: Token::EOF,
            peek_token: Token::EOF,
        };

        p.next_token();
        p.next_token();
        p
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.l.next_token();
    }

    pub fn parse_program(&mut self) -> ParserResult<Program> {
        let mut stmts = Program::new();
        while self.current_token != Token::EOF {
            let stmt = self.parse_stmt()?;
            stmts.0.push(stmt);
            self.next_token();
        }
        Ok(stmts)
    }

    fn parse_pattern(&mut self) -> ParserResult<Pattern> {
        let pattern = match self.current_token.clone() {
            Token::Ident(id) => {
                if id.as_str() == "_" {
                    Pattern::Nothing
                } else {
                    Pattern::Ident(Ident(id))
                }
            }
            Token::Symbol(sym) => Pattern::Symbol(sym),
            Token::String(str) => Pattern::String(str),
            Token::True => Pattern::Boolean(true),
            Token::False => Pattern::Boolean(false),
            Token::Number(num) => Pattern::Number(num),
            Token::LeftBracket => {
                let mut items = vec![];
                if self.peek_token_is(&Token::RightBracket) {
                    self.next_token();
                    self.next_token();
                    Pattern::Array(items)
                } else {
                    self.next_token();
                    let value = self.parse_pattern()?;
                    items.push(value);

                    while self.peek_token_is(&Token::Comma) {
                        self.next_token();
                        self.next_token();
                        let value = self.parse_pattern()?;
                        items.push(value);
                    }

                    // Read past the RightBracket
                    self.expect_peek(&Token::RightBracket, ParserType::Pattern)?;

                    Pattern::Array(items)
                }
            }
            Token::LeftBrace => {
                let mut items = vec![];
                if self.peek_token_is(&Token::RightBrace) {
                    self.next_token();
                    self.next_token();
                    Pattern::Hash(items)
                } else {
                    self.next_token();
                    let value = self.parse_identifier()?;
                    items.push((value, None));

                    while self.peek_token_is(&Token::Comma) {
                        self.next_token();
                        self.next_token();
                        let value = self.parse_identifier()?;
                        items.push((value, None));
                    }

                    // Read past the RightBracket
                    self.expect_peek(&Token::RightBrace, ParserType::Pattern)?;

                    Pattern::Hash(items)
                }
            }
            _ => return Err(String::new()),
        };

        Ok(pattern)
    }

    fn parse_stmt(&mut self) -> ParserResult<Stmt> {
        match self.current_token.clone() {
            Token::Let => self.parse_assign_stmt(),
            Token::Return => self.parse_return_stmt(),
            Token::Import => self.parse_import_stmt(),
            _ => self.parse_expression_stmt(),
        }
    }

    fn parse_assign_stmt(&mut self) -> ParserResult<Stmt> {
        self.next_token();
        let name = self.parse_pattern()?;
        // Should be an equals sign
        self.expect_peek(&Token::Assign, ParserType::Assign)?;

        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token_is(&Token::Semicolon) {
            self.next_token();
        }
        Ok(Stmt::Assign(name, value))
    }
    fn parse_return_stmt(&mut self) -> ParserResult<Stmt> {
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token_is(&Token::Semicolon) {
            self.next_token();
        }
        Ok(Stmt::Return(value))
    }
    fn parse_import_stmt(&mut self) -> ParserResult<Stmt> {
        self.next_token();
        let name = self.parse_pattern()?;
        self.expect_peek(&Token::From, ParserType::Import)?;
        self.next_token();
        let source = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token_is(&Token::Semicolon) {
            self.next_token();
        }
        Ok(Stmt::Import { source, name })
    }
    fn parse_expression_stmt(&mut self) -> ParserResult<Stmt> {
        let expression = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token_is(&Token::Semicolon) {
            self.next_token()
        }
        Ok(Stmt::Expr(expression))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> ParserResult<Expr> {
        let mut left = match self.current_token.clone() {
            Token::Ident(_) => self.parse_identifier().map(|v| Expr::Ident(v)),
            Token::String(_) => self.parse_string(),
            Token::Symbol(_) => self.parse_symbol(),
            Token::Number(_) => self.parse_number(),
            Token::Bang => self.parse_prefix_expression(),
            Token::Minus => self.parse_prefix_expression(),
            Token::True | Token::False => Ok(self.parse_boolean()),
            Token::LeftParen => self.parse_grouped_expressions(),
            Token::If => self.parse_if_expression(),
            Token::Function => self.parse_function(),
            Token::LeftBracket => self.parse_array(),
            Token::LeftBrace => self.parse_hash(),
            tok => {
                self.no_prefix_parser_error(&tok);
                return Err(format!("No prefix parser found for {}", &tok));
            }
        };
        while !self.peek_token_is(&Token::Semicolon) && precedence < self.peek_precedence() {
            left = match self.peek_token {
                Token::Plus
                | Token::Minus
                | Token::Range
                | Token::Asterisk
                | Token::Slash
                | Token::Modulus
                | Token::Eq
                | Token::NotEq
                | Token::Greater
                | Token::GreaterEq
                | Token::And
                | Token::Or
                | Token::Less
                | Token::LessEq => {
                    self.next_token();
                    self.parse_infix_expression(left.clone()?)
                }
                Token::LeftParen => {
                    self.next_token();
                    self.parse_call_expression(left.clone()?)
                }
                Token::Match => {
                    self.next_token();
                    self.parse_match(left.clone()?)
                }
                _ => return Ok(left?),
            };
        }
        Ok(left?)
    }

    fn parse_identifier(&mut self) -> ParserResult<Ident> {
        if let Token::Ident(ident) = self.current_token.clone() {
            return Ok(Ident(ident));
        }
        // This should never happen
        Err(String::new())
    }
    fn parse_number(&mut self) -> ParserResult<Expr> {
        if let Token::Number(num) = self.current_token.clone() {
            return Ok(Expr::Number(num));
        }
        // This should never happen
        Err("Expected number token".to_string())
    }
    fn parse_boolean(&mut self) -> Expr {
        let value = self.current_token_is(&Token::True);
        Expr::Boolean(value)
    }
    fn parse_string(&mut self) -> ParserResult<Expr> {
        if let Token::String(value) = self.current_token.clone() {
            return Ok(Expr::String(value));
        }
        Err("Expected \" or ' token".to_string())
    }
    fn parse_symbol(&mut self) -> ParserResult<Expr> {
        if let Token::Symbol(value) = self.current_token.clone() {
            return Ok(Expr::Symbol(value));
        }
        Err("Expected : token".to_string())
    }
    fn parse_prefix_expression(&mut self) -> ParserResult<Expr> {
        let operator = match self.current_token {
            Token::Bang => "!",
            Token::Minus => "-",
            _ => "",
        };
        self.next_token();
        // Hopefully this doesn't panic
        let right = self.parse_expression(Precedence::Prefix)?;
        Ok(Expr::Prefix(operator.to_string(), Box::new(right)))
    }
    fn parse_infix_expression(&mut self, left: Expr) -> ParserResult<Expr> {
        let operator = match self.current_token {
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Asterisk => "*",
            Token::Slash => "/",
            Token::Modulus => "%",
            Token::Eq => "==",
            Token::NotEq => "!=",
            Token::Greater => ">",
            Token::GreaterEq => ">=",
            Token::Less => "<",
            Token::LessEq => "<=",
            Token::Range => "..",
            Token::And => "&&",
            Token::Or => "||",
            _ => "",
        };

        let precedence = self.current_precedence();
        self.next_token();
        // I sure hope this doesn't panic
        let right = self.parse_expression(precedence)?;

        Ok(Expr::Infix(
            Box::new(left),
            operator.to_string(),
            Box::new(right),
        ))
    }
    fn parse_grouped_expressions(&mut self) -> ParserResult<Expr> {
        self.next_token();
        let exp = self.parse_expression(Precedence::Lowest)?;
        self.expect_peek(&Token::RightParen, ParserType::Grouped)?;
        Ok(exp)
    }
    fn parse_if_expression(&mut self) -> ParserResult<Expr> {
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;
        // TODO: Support shortened versions like this:
        //  if true then 5 else 10
        self.expect_peek(&Token::LeftBrace, ParserType::If)?;
        let consequence = self.parse_block_stmt()?;
        self.expect_peek(&Token::Else, ParserType::If)?;
        self.expect_peek(&Token::LeftBrace, ParserType::If)?;
        let alternative = self.parse_block_stmt()?;

        Ok(Expr::If {
            condition: Box::new(condition),
            consequence,
            alternative,
        })
    }
    fn parse_function(&mut self) -> ParserResult<Expr> {
        let parameters = match self.peek_token.clone() {
            Token::LeftParen => {
                self.next_token();
                self.parse_function_parameters()?
            }
            Token::Ident(ident) => {
                self.next_token();
                vec![Ident(ident)]
            }
            token => return Err(format!("Expected ( or identifier, found token {}", token)),
        };

        self.expect_peek(&Token::Arrow, ParserType::Function)?;
        self.next_token();

        let body = self.parse_block_shorthand()?;

        Ok(Expr::Function { parameters, body })
    }
    fn parse_function_parameters(&mut self) -> ParserResult<Vec<Ident>> {
        let mut identifiers = vec![];

        if self.peek_token_is(&Token::RightParen) {
            self.next_token();
            return Ok(identifiers);
        }

        self.next_token();

        let ident = match &self.current_token {
            Token::Ident(name) => Ident(name.clone()),
            _ => Ident(String::new()),
        };
        identifiers.push(ident);

        while self.peek_token_is(&Token::Comma) {
            self.next_token();
            self.next_token();
            let ident = match &self.current_token {
                Token::Ident(name) => Ident(name.clone()),
                _ => Ident(String::new()),
            };
            identifiers.push(ident);
        }

        self.expect_peek(&Token::RightParen, ParserType::Call)?;

        Ok(identifiers)
    }
    fn parse_call_expression(&mut self, function: Expr) -> ParserResult<Expr> {
        let args = self.parse_call_expression_args()?;
        Ok(Expr::Call {
            function: Box::new(function),
            arguments: args,
        })
    }
    fn parse_call_expression_args(&mut self) -> ParserResult<Vec<Expr>> {
        let mut args = vec![];

        if self.peek_token_is(&Token::RightParen) {
            self.next_token();
            return Ok(args);
        }

        self.next_token();

        args.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token_is(&Token::Comma) {
            self.next_token();
            self.next_token();
            args.push(self.parse_expression(Precedence::Lowest)?);
        }

        self.expect_peek(&Token::RightParen, ParserType::Call)?;

        Ok(args)
    }

    fn parse_block_stmt(&mut self) -> ParserResult<BlockStatement> {
        let mut stmts = vec![];
        self.next_token();
        // self.next_token();
        while !self.current_token_is(&Token::RightBrace) && !self.current_token_is(&Token::EOF) {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
            self.next_token();
        }

        Ok(BlockStatement(stmts))
    }

    fn parse_block_shorthand(&mut self) -> ParserResult<BlockStatement> {
        if self.current_token_is(&Token::LeftBrace) {
            return self.parse_block_stmt();
        }
        let expr = self.parse_expression(Precedence::Lowest)?;
        Ok(Stmt::Expr(expr).into())
    }

    fn parse_match(&mut self, condition: Expr) -> ParserResult<Expr> {
        self.expect_peek(&Token::LeftBrace, ParserType::Match)?;
        let cases = self.parse_match_cases()?;
        self.expect_peek(&Token::RightBrace, ParserType::Match)?;
        Ok(Expr::Match {
            condition: Box::new(condition),
            cases,
        })
    }
    fn parse_match_cases(&mut self) -> ParserResult<Vec<(Pattern, BlockStatement)>> {
        let mut cases = vec![];

        if self.peek_token_is(&Token::RightBrace) {
            return Ok(cases);
        }
        self.next_token();
        let key = self.parse_pattern()?;
        self.expect_peek(&Token::Arrow, ParserType::Match)?;
        self.next_token();
        let value = self.parse_block_shorthand()?;

        cases.push((key, value));

        while self.peek_token_is(&Token::Comma) {
            self.next_token();
            self.next_token();
            let key = self.parse_pattern()?;
            self.expect_peek(&Token::Arrow, ParserType::Match)?;
            self.next_token();
            let value = self.parse_block_shorthand()?;

            cases.push((key, value));
        }
        Ok(cases)
    }

    fn parse_array(&mut self) -> ParserResult<Expr> {
        let mut items = vec![];
        if self.peek_token_is(&Token::RightBracket) {
            self.next_token();
            self.next_token();
            return Ok(Expr::Array(items));
        }

        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;
        items.push(value);

        while self.peek_token_is(&Token::Comma) {
            self.next_token();
            self.next_token();
            let value = self.parse_expression(Precedence::Lowest)?;
            items.push(value);
        }

        // Read past the RightBracket
        self.expect_peek(&Token::RightBracket, ParserType::Array)?;

        Ok(Expr::Array(items))
    }
    fn parse_hash(&mut self) -> ParserResult<Expr> {
        let mut items = vec![];
        if self.peek_token_is(&Token::RightBrace) {
            self.next_token();
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
                if self.peek_token_is(&Token::Comma) || self.peek_token_is(&Token::RightBrace) {
                    items.push((ident, key))
                } else {
                    self.expect_peek(&Token::Assign, ParserType::Hash)?;
                    self.next_token();
                    let value = self.parse_expression(Precedence::Lowest)?;
                    items.push((ident, value));
                }
            }
            _ => return Ok(Expr::Hash(items)),
        }

        while self.peek_token_is(&Token::Comma) {
            self.next_token();
            self.next_token();
            let key = self.parse_expression(Precedence::Lowest)?;
            match key.clone() {
                Expr::Ident(ident) => {
                    if self.peek_token_is(&Token::Comma) {
                        items.push((ident, key))
                    } else {
                        self.expect_peek(&Token::Assign, ParserType::Hash)?;
                        self.next_token();
                        let value = self.parse_expression(Precedence::Lowest)?;
                        items.push((ident, value));
                    }
                }
                _ => return Ok(Expr::Hash(items)),
            }
        }

        // Read past the RightBracket
        self.expect_peek(&Token::RightBrace, ParserType::Hash)?;

        Ok(Expr::Hash(items))
    }

    // Utils
    fn current_token_is(&mut self, t: &Token) -> bool {
        &self.current_token == t
    }
    fn peek_token_is(&mut self, t: &Token) -> bool {
        &self.peek_token == t
    }
    fn expect_peek(&mut self, t: &Token, context: ParserType) -> Result<(), String> {
        if self.peek_token_is(t) {
            self.next_token();
            return Ok(());
        }
        Err(self.peek_error(&t, context))
    }
    fn peek_precedence(&mut self) -> Precedence {
        get_precedence(&self.peek_token)
    }
    fn current_precedence(&mut self) -> Precedence {
        get_precedence(&self.current_token)
    }
    // Errors stuff
    fn peek_error(&mut self, t: &Token, context: ParserType) -> String {
        // TODO: Make better error handling
        let attempted_msg = generate_parser_message(
            ParserError::ExpectedFound(t, &self.peek_token),
            context.clone(),
        );
        let mut msg = format!(
            "Expected next token to be {:?}, got {:?} instead. This was in the {:?} parser",
            t, self.peek_token, context
        );
        if attempted_msg != String::new() {
            msg = attempted_msg;
        }
        msg
    }
    fn no_prefix_parser_error(&mut self, t: &Token) -> String {
        let msg = format!("No prefix parse function for {:?} found", t);
        msg
    }

    // fn debug(&self) {
    //     println!(
    //         "The current token is {:?}, and the peek is {:?}",
    //         self.current_token, self.peek_token
    //     );
    // }
}
