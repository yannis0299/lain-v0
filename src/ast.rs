use crate::tokenizer::{Token, TokenKind};
use eyre::{ContextCompat, Result, bail};
use std::mem;

// High-level syntax tree
#[derive(Debug, Clone)]
pub enum AST {
    Empty,                                     // no atoms in application buffer
    Unit,                                      // ()
    Integer(i64),                              // integer literal
    Character(char),                           // character literal
    String(String),                            // string literal
    Variable(String),                          // variable
    Wildcard,                                  // _
    Tuple(Box<AST>, Vec<AST>, Box<AST>),       // (e1, .., eN)
    List(Vec<AST>),                            // [e1, .., eN]
    Lambda(String, Vec<String>, Box<AST>),     // \x1 .. xn => e
    Application(Box<AST>, Vec<AST>, Box<AST>), // (e1 .. eN)
    Match(Box<AST>, Box<(AST, AST)>, Vec<(AST, AST)>),
    Bindings(Vec<(String, Vec<String>, Box<AST>)>),
    Where(Vec<(String, Vec<String>, AST)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    AST_LIST,
    TUPLE,
    LIST,
    LAMBDA,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub kind: FrameKind,
    pub stack: Vec<AST>,
}

impl Frame {
    pub fn new(kind: FrameKind) -> Self {
        Self {
            kind,
            stack: vec![],
        }
    }
}

pub struct ASTBuilder<'a> {
    pub tokens: Vec<Token<'a>>,
    pub frame_stack: Vec<Frame>,
    pub app_buffer: Vec<AST>,
}

impl<'a> ASTBuilder<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens: tokens.into_iter().rev().collect(),
            frame_stack: vec![Frame::new(FrameKind::AST_LIST)],
            app_buffer: vec![],
        }
    }

    pub fn push_new_frame(&mut self, kind: FrameKind) {
        self.frame_stack.push(Frame::new(kind));
    }

    pub fn push_to_top_frame<P: Fn(FrameKind) -> bool + 'static>(
        &mut self,
        expecting: P,
        ast: AST,
    ) -> Result<()> {
        match self.frame_stack.last_mut() {
            None => bail!("ASTBuilder: Attempting to push an AST to an empty stack frame"),
            Some(frame) => {
                if expecting(frame.kind) {
                    frame.stack.push(ast);
                    Ok(())
                } else {
                    bail!(
                        "ASTBuilder: Attempting to push an AST to an invalid stack frame {:?}\n  Probably because of a mismatch of opening/closing delimiter or a malformed expression",
                        frame.kind
                    )
                }
            }
        }
    }

    pub fn pop_top_frame<P: Fn(FrameKind) -> bool + 'static>(
        &mut self,
        expecting: P,
    ) -> Result<Frame> {
        match self.frame_stack.last() {
            None => bail!("ASTBuilder: Attempting to push an AST to an empty stack frame"),
            Some(frame) => {
                if expecting(frame.kind) {
                    self.frame_stack.pop().wrap_err("unreachable")
                } else {
                    bail!(
                        "ASTBuilder: Attempting to push an AST to an invalid stack frame {:?}\n  Probably because of a mismatch of opening/closing delimiter or a malformed expression",
                        frame.kind
                    )
                }
            }
        }
    }

    pub fn push_to_app_buffer(&mut self, ast: AST) {
        self.app_buffer.push(ast);
    }

    pub fn reduce_app_buffer(&mut self) -> Result<AST> {
        let ast = {
            let mut app_buffer = vec![];
            mem::swap(&mut self.app_buffer, &mut app_buffer);
            match app_buffer.len() {
                0 => AST::Empty,
                1 => app_buffer.into_iter().next().wrap_err("unreachable")?,
                _ => {
                    let mut iter = app_buffer.into_iter();
                    let h = iter.next().wrap_err("unreachable")?;
                    let mut m = iter.collect::<Vec<_>>();
                    let t = m.pop().wrap_err("unreachable")?;
                    AST::Application(Box::new(h), m, Box::new(t))
                }
            }
        };
        Ok(ast)
    }

    pub fn step(&mut self) -> Result<bool> {
        if let Some(token) = self.tokens.pop() {
            match token.kind {
                TokenKind::Underscore => {
                    self.push_to_app_buffer(AST::Wildcard);
                }
                TokenKind::Integer => {
                    let value = token.repr.parse()?;
                    self.push_to_app_buffer(AST::Integer(value));
                }
                TokenKind::Character => {
                    let value = token.repr[1..token.repr.len() - 1].parse()?;
                    self.push_to_app_buffer(AST::Character(value));
                }
                TokenKind::String => {
                    let value = token.repr[1..token.repr.len() - 1].parse()?;
                    self.push_to_app_buffer(AST::String(value));
                }
                TokenKind::Identifier => {
                    let value = String::from(token.repr);
                    self.push_to_app_buffer(AST::Variable(value));
                }
                TokenKind::Comma => {
                    let ast = self.reduce_app_buffer()?;
                    self.push_to_top_frame(
                        |kind| kind == FrameKind::TUPLE || kind == FrameKind::LIST,
                        ast,
                    )?;
                }
                TokenKind::LeftParen => self.push_new_frame(FrameKind::TUPLE),
                TokenKind::RightParen => {
                    let mut top_frame = self.pop_top_frame(|kind| kind == FrameKind::TUPLE)?;
                    let ast = self.reduce_app_buffer()?;
                    match ast {
                        AST::Empty => (),
                        _ => top_frame.stack.push(ast),
                    }
                    let atom = {
                        match top_frame.stack.len() {
                            0 => AST::Unit,
                            1 => top_frame.stack.into_iter().next().wrap_err("unreachable")?,
                            _ => {
                                let mut iter = top_frame.stack.into_iter();
                                let h = iter.next().wrap_err("unreachable")?;
                                let mut m = iter.collect::<Vec<_>>();
                                let t = m.pop().wrap_err("unreachable")?;
                                AST::Tuple(Box::new(h), m, Box::new(t))
                            }
                        }
                    };
                    self.app_buffer.push(atom);
                }
                TokenKind::LeftBracket => self.push_new_frame(FrameKind::LIST),
                TokenKind::RightBracket => {
                    let mut top_frame = self.pop_top_frame(|kind| kind == FrameKind::LIST)?;
                    let ast = self.reduce_app_buffer()?;
                    match ast {
                        AST::Empty => (),
                        _ => top_frame.stack.push(ast),
                    }
                    let atom = AST::List(top_frame.stack);
                    self.app_buffer.push(atom);
                }
                _ => (), // skip for now
            };
            return Ok(true);
        }
        Ok(false)
    }

    pub fn exhaust(&mut self) -> Result<()> {
        loop {
            if !self.step()? {
                break;
            }
        }
        let ast = self.reduce_app_buffer()?;
        self.push_to_top_frame(|kind| kind == FrameKind::AST_LIST, ast)
    }
}
