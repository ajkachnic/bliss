use ast::{BlockStatement, Expr, Ident, Program, Stmt};
use error::{
    generate_parser_message, generate_pretty_error, ParserError, ParserResult, ParserType,
};

use crate::lexer::Lexer;
use crate::token::{Token, TokenType};
use crate::{ast, token::Position};

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

    pub fn parse_program(&mut self) -> ParserResult<Program> {
        let mut stmts = Program::new();
        while self.current_token.tok != TokenType::EOF {
            let stmt = self.parse_stmt()?;
            stmts.0.push(stmt);
            self.next_token();
        }
        Ok(stmts)
    }
    fn parse_stmt(&mut self) -> ParserResult<Stmt> {
        match self.current_token.clone().tok {
            TokenType::Ident(_) => {
                if self.peek_token_is(&TokenType::Assign) {
                    let name = self.parse_expression(Precedence::Lowest)?;
                    return self.parse_assign_stmt(name);
                }
                self.parse_expression_stmt()
            }
            TokenType::LeftBrace => {
                let name = self.parse_expression(Precedence::Lowest)?;
                if self.peek_token_is(&TokenType::Assign) {
                    return self.parse_assign_stmt(name);
                }
                if self.peek_token_is(&TokenType::Semicolon) {
                    self.next_token()
                }
                Ok(Stmt::Expr(name))
            }
            TokenType::LeftBracket => {
                let name = self.parse_expression(Precedence::Lowest)?;
                if self.peek_token_is(&TokenType::Assign) {
                    return self.parse_assign_stmt(name);
                }
                if self.peek_token_is(&TokenType::Semicolon) {
                    self.next_token()
                }
                Ok(Stmt::Expr(name))
            }
            TokenType::Return => self.parse_return_stmt(),
            TokenType::Import => self.parse_import_stmt(),
            _ => self.parse_expression_stmt(),
        }
    }

    fn parse_assign_stmt(&mut self, name: Expr) -> ParserResult<Stmt> {
        // Should be an equals sign
        self.expect_peek(&TokenType::Assign, ParserType::Assign)?;

        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        Ok(Stmt::Assign(name, value))
    }
    fn parse_return_stmt(&mut self) -> ParserResult<Stmt> {
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        Ok(Stmt::Return(value))
    }
    fn parse_import_stmt(&mut self) -> ParserResult<Stmt> {
        self.next_token();
        let name = self.parse_expression(Precedence::Lowest)?;
        self.expect_peek(&TokenType::From, ParserType::Import)?;
        self.next_token();
        let source = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token();
        }
        Ok(Stmt::Import { source, name })
    }
    fn parse_expression_stmt(&mut self) -> ParserResult<Stmt> {
        let expression = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token_is(&TokenType::Semicolon) {
            self.next_token()
        }
        Ok(Stmt::Expr(expression))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> ParserResult<Expr> {
        let token = self.current_token.clone();
        let mut left = match token.tok {
            TokenType::Ident(_) => self.parse_identifier(),
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
                    self.parse_infix_expression(left.clone()?)
                }
                TokenType::LeftParen => {
                    self.next_token();
                    self.parse_call_expression(left.clone()?)
                }
                TokenType::Match => {
                    self.next_token();
                    self.parse_match(left.clone()?)
                }
                _ => return Ok(left?),
            };
        }
        Ok(left?)
    }

    fn parse_identifier(&mut self) -> ParserResult<Expr> {
        if let TokenType::Ident(ident) = self.current_token.clone().tok {
            return Ok(Expr::Ident(Ident(ident)));
        }
        // This should never happen
        Err(String::new())
    }
    fn parse_number(&mut self) -> ParserResult<Expr> {
        // TODO: Find a better way to do this without disturbing the range operator (..)
        // Really really cursed code to generate floats
        if let TokenType::Number(num) = self.current_token.clone().tok {
            let mut value = String::from(&num);
            if self.peek_token_is(&TokenType::Period) {
                self.next_token();
                if let TokenType::Number(num) = self.peek_token.clone().tok {
                    self.next_token();
                    value.push('.');
                    value.push_str(&num);
                }
            }
            let parsed = value.parse();
            if let Ok(num) = parsed {
                return Ok(Expr::Number(num));
            }
            return Err(format!("Failed to parse number {} as a float", &num));
        }
        // This should never happen
        Err("Expected number token".to_string())
    }
    fn parse_boolean(&mut self) -> Expr {
        let value = self.current_token_is(&TokenType::True);
        Expr::Boolean(value)
    }
    fn parse_string(&mut self) -> ParserResult<Expr> {
        if let TokenType::String(value) = self.current_token.clone().tok {
            return Ok(Expr::String(value));
        }
        Err("Expected \" or ' token".to_string())
    }
    fn parse_symbol(&mut self) -> ParserResult<Expr> {
        if let TokenType::Symbol(value) = self.current_token.clone().tok {
            return Ok(Expr::Symbol(value));
        }
        Err("Expected : token".to_string())
    }
    fn parse_prefix_expression(&mut self) -> ParserResult<Expr> {
        let operator = match self.current_token.tok {
            TokenType::Bang => "!",
            TokenType::Minus => "-",
            _ => "",
        };
        self.next_token();
        let right = self.parse_expression(Precedence::Prefix)?;
        Ok(Expr::Prefix(operator.to_string(), Box::new(right)))
    }
    fn parse_infix_expression(&mut self, left: Expr) -> ParserResult<Expr> {
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
        self.expect_peek(&TokenType::RightParen, ParserType::Grouped)?;
        Ok(exp)
    }
    fn parse_if_expression(&mut self) -> ParserResult<Expr> {
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;
        // TODO: Support shortened versions like this:
        //  if true then 5 else 10
        self.expect_peek(&TokenType::LeftBrace, ParserType::If)?;
        let consequence = self.parse_block_stmt()?;
        self.expect_peek(&TokenType::Else, ParserType::If)?;
        self.expect_peek(&TokenType::LeftBrace, ParserType::If)?;
        let alternative = self.parse_block_stmt()?;

        Ok(Expr::If {
            condition: Box::new(condition),
            consequence,
            alternative,
        })
    }
    fn parse_function(&mut self) -> ParserResult<Expr> {
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
            _ => return Err(format!("Expected ( or identifier, found token {:?}", token)),
        };

        self.expect_peek(&TokenType::Arrow, ParserType::Function)?;
        self.next_token();

        let body = self.parse_block_shorthand()?;

        Ok(Expr::Function { parameters, body })
    }
    fn parse_function_parameters(&mut self) -> ParserResult<Vec<Ident>> {
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

        self.expect_peek(&TokenType::RightParen, ParserType::Call)?;

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

        self.expect_peek(&TokenType::RightParen, ParserType::Call)?;

        Ok(args)
    }

    fn parse_block_stmt(&mut self) -> ParserResult<BlockStatement> {
        let mut stmts = vec![];
        self.next_token();
        // self.next_token();
        while !self.current_token_is(&TokenType::RightBrace)
            && !self.current_token_is(&TokenType::EOF)
        {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
            self.next_token();
        }

        Ok(BlockStatement(stmts))
    }

    fn parse_block_shorthand(&mut self) -> ParserResult<BlockStatement> {
        if self.current_token_is(&TokenType::LeftBrace) {
            return self.parse_block_stmt();
        }
        let expr = self.parse_expression(Precedence::Lowest)?;
        Ok(Stmt::Expr(expr).into())
    }

    fn parse_match(&mut self, condition: Expr) -> ParserResult<Expr> {
        self.expect_peek(&TokenType::LeftBrace, ParserType::Match)?;
        let cases = self.parse_match_cases()?;
        self.expect_peek(&TokenType::RightBrace, ParserType::Match)?;
        Ok(Expr::Match {
            condition: Box::new(condition),
            cases,
        })
    }
    fn parse_match_cases(&mut self) -> ParserResult<Vec<(Expr, BlockStatement)>> {
        let mut cases = vec![];

        if self.peek_token_is(&TokenType::RightBrace) {
            return Ok(cases);
        }
        self.next_token();
        let key = self.parse_expression(Precedence::Lowest)?;
        self.expect_peek(&TokenType::Arrow, ParserType::Match)?;
        self.next_token();
        let value = self.parse_block_shorthand()?;

        cases.push((key, value));

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            let key = self.parse_expression(Precedence::Lowest)?;
            self.expect_peek(&TokenType::Arrow, ParserType::Match)?;
            self.next_token();
            let value = self.parse_block_shorthand()?;

            cases.push((key, value));
        }
        Ok(cases)
    }

    fn parse_array(&mut self) -> ParserResult<Expr> {
        let mut items = vec![];
        if self.peek_token_is(&TokenType::RightBracket) {
            self.next_token();
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
        self.expect_peek(&TokenType::RightBracket, ParserType::Array)?;

        Ok(Expr::Array(items))
    }
    fn parse_hash(&mut self) -> ParserResult<Expr> {
        let mut items = vec![];
        if self.peek_token_is(&TokenType::RightBrace) {
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
                if self.peek_token_is(&TokenType::Comma)
                    || self.peek_token_is(&TokenType::RightBrace)
                {
                    items.push((ident, key))
                } else {
                    self.expect_peek(&TokenType::Assign, ParserType::Hash)?;
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
                        self.expect_peek(&TokenType::Assign, ParserType::Hash)?;
                        self.next_token();
                        let value = self.parse_expression(Precedence::Lowest)?;
                        items.push((ident, value));
                    }
                }
                _ => return Ok(Expr::Hash(items)),
            }
        }

        // Read past the RightBracket
        self.expect_peek(&TokenType::RightBrace, ParserType::Hash)?;

        Ok(Expr::Hash(items))
    }

    // Utils
    fn current_token_is(&mut self, t: &TokenType) -> bool {
        &self.current_token.tok == t
    }
    fn peek_token_is(&mut self, t: &TokenType) -> bool {
        &self.peek_token.tok == t
    }
    fn expect_peek(&mut self, t: &TokenType, context: ParserType) -> Result<(), String> {
        if self.peek_token_is(t) {
            self.next_token();
            return Ok(());
        }
        Err(self.peek_error(&t, context))
    }
    fn peek_precedence(&mut self) -> Precedence {
        get_precedence(&self.peek_token.tok)
    }
    fn current_precedence(&mut self) -> Precedence {
        get_precedence(&self.current_token.tok)
    }
    // Errors stuff
    fn peek_error(&mut self, t: &TokenType, context: ParserType) -> String {
        let t = Token {
            tok: t.clone(),
            offset: self.peek_token.offset,
        };
        let position = Position::from(t.offset, &self.source);

        let attempted_msg = generate_parser_message(
            ParserError::ExpectedFound(&t.tok, &self.peek_token.tok, position.clone()),
            context.clone(),
            position.clone(),
            &self.source,
        );
        let mut msg = format!(
            "On line {}, and column {}, we expected next token to be {}, got {} instead. This was in the {:?} parser", position.line, position.column,
            t.tok, self.peek_token.tok, context
        );
        if attempted_msg != String::new() {
            msg = attempted_msg;
        }
        msg
    }
    fn no_prefix_parser_error(&mut self, t: Token) -> String {
        let position = Position::from(self.current_token.offset, self.source.as_str());
        let mut msg = format!(
            "On {}, no prefix parse function for {} found",
            position, t.tok
        );
        let error = generate_pretty_error(position, &self.source);
        msg.push('\n');
        msg.push_str(&error);

        msg
    }

    // fn debug(&self) {
    //     println!(
    //         "The current token is {:?}, and the peek is {:?}",
    //         self.current_token, self.peek_token
    //     );
    // }
}
