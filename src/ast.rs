use std::collections::HashMap;

use crate::lexer::{Token, TokenKind};

use eyre::{bail, eyre, ContextCompat, Result};
use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub enum RawAST {
    // Leaf
    Empty,
    // Atoms
    Unit,                  // ()
    Integer(i32),          // k
    Wildcard,              // _
    Variable(String),      // ident
    Tuple(AST, ASTs, AST), // (e1, .., eN) at least 0 elements
    List(ASTs),            // [e1, .., eN] can have 0 elements
    // Composite expression
    Lambda(AST, AST),            // \e1 => e2
    Application(AST, ASTs, AST), // e1 .. eN
    IfThenElse(AST, AST, AST),   // if e1 then e2 else e3
    Operator(String, AST, AST),  // e1 `op` e2
}

#[allow(clippy::upper_case_acronyms)]
pub type AST = Box<RawAST>;
pub type ASTs = Vec<RawAST>;

#[derive(Debug, Clone, Copy)]
pub enum OperatorAssoc {
    InfixLeft,
    InfixRight,
}

lazy_static! {
    static ref OPERATORS: HashMap<&'static str, (OperatorAssoc, i32)> = vec![
        ("+", (OperatorAssoc::InfixLeft, 6)),
        ("-", (OperatorAssoc::InfixLeft, 6)),
        ("*", (OperatorAssoc::InfixLeft, 7)),
        ("/", (OperatorAssoc::InfixLeft, 7)),
        ("$", (OperatorAssoc::InfixRight, 0)),
        (".", (OperatorAssoc::InfixRight, 9)),
        ("::", (OperatorAssoc::InfixRight, 5))
    ]
    .into_iter()
    .collect();
}

pub struct ExprParser<'a> {
    pub contents: &'a str,
    pub pos: usize,
    pub tokens: &'a [Token],
}

impl<'a> ExprParser<'a> {
    pub fn new(contents: &'a str, tokens: &'a [Token]) -> Self {
        Self {
            contents,
            pos: 0,
            tokens,
        }
    }

    pub fn parse(mut self) -> Result<RawAST> {
        let expr = self.parse_expr(0)?;
        Ok(expr)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        self.pos += 1;
        t
    }

    fn expect(&mut self, kind: TokenKind) -> Result<()> {
        match self.next() {
            Some(t) if t.kind == kind => Ok(()),
            _ => bail!("ExprParser: Expected {:?}", kind),
        }
    }

    fn parse_expr(&mut self, min_prec: i32) -> Result<RawAST> {
        match self.peek() {
            None => Ok(RawAST::Empty),
            Some(token)
                if matches!(
                    token.kind,
                    TokenKind::Integer
                        | TokenKind::Underscore
                        | TokenKind::Identifier
                        | TokenKind::LeftParen
                        | TokenKind::LeftBracket
                ) =>
            {
                let mut lhs = self.parse_atoms()?;
                loop {
                    let op_token = match self.peek() {
                        Some(t) if matches!(t.kind, TokenKind::Operator) => t,
                        _ => break,
                    };
                    let op_str = &self.contents[op_token.span.0..op_token.span.1];
                    let (assoc, prec) = match OPERATORS.get(op_str) {
                        Some(v) => *v,
                        None => bail!("ExprParser: Unknown operator {:?}", op_str),
                    };
                    if prec < min_prec {
                        break;
                    }
                    self.next(); // consume operator
                    let next_min_prec = match assoc {
                        OperatorAssoc::InfixLeft => prec + 1,
                        OperatorAssoc::InfixRight => prec,
                    };
                    let rhs = self.parse_expr(next_min_prec)?;
                    lhs = RawAST::Operator(op_str.to_string(), Box::new(lhs), Box::new(rhs));
                }
                Ok(lhs)
            }
            Some(token) if matches!(token.kind, TokenKind::If | TokenKind::Backslash) => {
                let token = self.next().wrap_err("unreachable")?;
                match token.kind {
                    TokenKind::If => {
                        let cond = self.parse_expr(0)?;
                        self.expect(TokenKind::Then)?;
                        let then_branch = self.parse_expr(0)?;
                        self.expect(TokenKind::Else)?;
                        let else_branch = self.parse_expr(0)?;
                        Ok(RawAST::IfThenElse(
                            Box::new(cond),
                            Box::new(then_branch),
                            Box::new(else_branch),
                        ))
                    }
                    TokenKind::Backslash => {
                        let param = self.parse_atoms()?;
                        self.expect(TokenKind::RightFatArrow)?;
                        let body = self.parse_expr(0)?;
                        Ok(RawAST::Lambda(Box::new(param), Box::new(body)))
                    }
                    _ => bail!("unreachable"),
                }
            }
            Some(token) if matches!(token.kind, TokenKind::Operator) => {
                let mut lhs = RawAST::Empty;
                let op_token = self.next().wrap_err("unreachable")?.clone();
                let op_str = &self.contents[op_token.span.0..op_token.span.1];
                let rhs = self.parse_atoms()?;
                lhs = RawAST::Operator(op_str.to_string(), Box::new(lhs), Box::new(rhs));
                Ok(lhs)
            }
            _token => Ok(RawAST::Empty),
        }
    }

    fn parse_atoms(&mut self) -> Result<RawAST> {
        let mut acc = vec![];
        loop {
            match self.peek() {
                Some(token)
                    if matches!(
                        token.kind,
                        TokenKind::Integer
                            | TokenKind::Identifier
                            | TokenKind::Underscore
                            | TokenKind::LeftParen
                            | TokenKind::LeftBracket
                    ) =>
                {
                    let atom = self.parse_atom()?;
                    acc.push(atom);
                }
                _ => break,
            }
        }
        match acc.len() {
            0 => Ok(RawAST::Empty),
            1 => acc.pop().wrap_err("unreachable"),
            _ => {
                let last = acc.pop().wrap_err("unreachable")?;
                let tail = acc.split_off(1);
                let head = acc.pop().wrap_err("unreachable")?;
                Ok(RawAST::Application(Box::new(head), tail, Box::new(last)))
            }
        }
    }

    fn parse_atom(&mut self) -> Result<RawAST> {
        let token = self.next().wrap_err("ExprParser: Unexpected EOF")?.clone();
        match token.kind {
            TokenKind::Integer => {
                let s = &self.contents[token.span.0..token.span.1];
                let value = s.parse()?;
                Ok(RawAST::Integer(value))
            }
            TokenKind::Identifier => {
                let s = &self.contents[token.span.0..token.span.1];
                let value = s.to_string();
                Ok(RawAST::Variable(value))
            }
            TokenKind::Underscore => Ok(RawAST::Wildcard),
            TokenKind::LeftParen => {
                let mut acc = vec![];
                loop {
                    let expr = self.parse_expr(0)?;
                    acc.push(expr);
                    match self.next() {
                        Some(t) if matches!(t.kind, TokenKind::RightParen) => break,
                        Some(t) if matches!(t.kind, TokenKind::Comma) => continue,
                        _ => bail!("ExprParser: Expected ')' or ',' while accumulating a tuple"),
                    }
                }
                match acc.len() {
                    0 => Ok(RawAST::Unit),
                    1 => match acc.pop().wrap_err("unreachable")? {
                        RawAST::Empty => Ok(RawAST::Unit),
                        ast => Ok(ast),
                    },
                    _ => {
                        let last = acc.pop().wrap_err("unreachable")?;
                        let tail = acc.split_off(1);
                        let head = acc.pop().wrap_err("unreachable")?;
                        Ok(RawAST::Tuple(Box::new(head), tail, Box::new(last)))
                    }
                }
            }
            TokenKind::LeftBracket => {
                let mut acc = vec![];
                loop {
                    let expr = self.parse_expr(0)?;
                    acc.push(expr);
                    match self.next() {
                        Some(t) if matches!(t.kind, TokenKind::RightBracket) => break,
                        Some(t) if matches!(t.kind, TokenKind::Comma) => continue,
                        _ => bail!("ExprParser: Expected ']' or ',' while accumulating a tuple"),
                    }
                }
                Ok(RawAST::List(acc))
            }
            _ => bail!("ExprParser: ExprParser: Unexpected token {:?}", token.kind),
        }
    }
}
