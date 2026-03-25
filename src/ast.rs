use std::mem;

use crate::lexer::{Token, TokenKind};

use eyre::{bail, ContextCompat, Result};

#[derive(Debug, Clone)]
pub enum RawAST {
    // Leaf
    Empty,
    // Atoms
    Unit,                  // ()
    Integer(i32),          // k
    Wildcard,              // _
    Variable(String),      // ident
    Tuple(AST, ASTs, AST), // (e1, .., eN) at least 2 elements
    List(ASTs),            // [e1, .., eN] can have 0 elements
    // Composite expression
    Application(AST, ASTs, AST),
}

pub type AST = Box<RawAST>;
pub type ASTs = Vec<RawAST>;

#[derive(Debug, Clone)]
pub enum Frame {
    AtomList,
    TupleAcc(ASTs),
    ListAcc(ASTs),
}

pub struct ASTBuilder<'a> {
    pub contents: &'a str,
    pub frame_stack: Vec<(Frame, ASTs)>,
}

impl<'a> ASTBuilder<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self {
            contents,
            frame_stack: vec![(Frame::AtomList, vec![])],
        }
    }

    pub fn push_atom(&mut self, atom: RawAST) -> Result<()> {
        let (_, app_buffer) = self.last_frame_mut()?;
        app_buffer.push(atom);
        Ok(())
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.frame_stack.push((frame, vec![]));
    }

    pub fn reduce_app_buffer(mut app_buffer: ASTs) -> Result<RawAST> {
        let ast = match app_buffer.len() {
            0 => RawAST::Empty,
            1 => app_buffer.pop().wrap_err("unreachable")?,
            _ => {
                let last = app_buffer.pop().wrap_err("unreachable")?;
                let mut app_buffer = app_buffer.into_iter().rev().collect::<Vec<_>>();
                let head = app_buffer.pop().wrap_err("unreachable")?;
                RawAST::Application(Box::new(head), app_buffer, Box::new(last))
            }
        };
        Ok(ast)
    }

    pub fn last_frame_mut(&mut self) -> Result<&mut (Frame, ASTs)> {
        self.frame_stack
            .last_mut()
            .wrap_err("ASTBuilder: attempting to reference last frame in an empty stack")
    }

    pub fn eat_tokens(&mut self, tokens: &[Token]) -> Result<()> {
        for token in tokens {
            println!("token = {:?}", token);
            println!("frame_stack = {:#?}", self.frame_stack);
            match token.kind {
                TokenKind::Underscore => {
                    self.push_atom(RawAST::Wildcard)?;
                }
                TokenKind::Integer => {
                    let repr = &self.contents[token.span.0..token.span.1];
                    let value = repr.parse()?;
                    self.push_atom(RawAST::Integer(value))?;
                }
                TokenKind::Identifier => {
                    let repr = &self.contents[token.span.0..token.span.1];
                    let value = String::from(repr);
                    self.push_atom(RawAST::Variable(value))?;
                }
                TokenKind::LeftParen => {
                    let frame = Frame::TupleAcc(vec![]);
                    self.push_frame(frame);
                }
                TokenKind::LeftBracket => {
                    let frame = Frame::ListAcc(vec![]);
                    self.push_frame(frame);
                }
                TokenKind::Comma => match self.last_frame_mut()? {
                    (Frame::TupleAcc(acc), app_buffer) | (Frame::ListAcc(acc), app_buffer) => {
                        let mut old_buffer = vec![];
                        mem::swap(app_buffer, &mut old_buffer);
                        let ast = Self::reduce_app_buffer(old_buffer)?;
                        acc.push(ast);
                    }
                    _ => bail!("ASTBuilder: Encountering comma in a none tuple or list expression"),
                },
                TokenKind::RightParen => match self.frame_stack.pop() {
                    Some((Frame::TupleAcc(mut acc), app_buffer)) => {
                        let ast = Self::reduce_app_buffer(app_buffer)?;
                        match ast {
                            RawAST::Empty => (),
                            _ => acc.push(ast),
                        }
                        let atom = match acc.len() {
                            0 => RawAST::Unit,
                            1 => acc.pop().wrap_err("unreachable")?,
                            _ => {
                                let mut tail = acc.split_off(1);
                                let head = acc.pop().wrap_err("unreachable")?;
                                let last = tail.pop().wrap_err("unreachable")?;
                                RawAST::Tuple(Box::new(head), tail, Box::new(last))
                            }
                        };
                        self.push_atom(atom)?;
                    }
                    _ => {
                        bail!("ASTBuilder: Attempting to close a tuple with an opening parethesis")
                    }
                },
                TokenKind::RightBracket => match self.frame_stack.pop() {
                    Some((Frame::ListAcc(mut acc), app_buffer)) => {
                        let ast = Self::reduce_app_buffer(app_buffer)?;
                        match ast {
                            RawAST::Empty => (),
                            _ => acc.push(ast),
                        }
                        let atom = RawAST::List(acc);
                        self.push_atom(atom)?;
                    }
                    _ => {
                        bail!("ASTBuilder: Attempting to close a list with an opening bracket")
                    }
                },
                TokenKind::If => todo!(),
                TokenKind::Then => todo!(),
                TokenKind::Else => todo!(),
                TokenKind::Backslash => todo!(),
                TokenKind::RightFatArrow => todo!(),
                TokenKind::Operator => todo!(),
                TokenKind::Match => todo!(),
                TokenKind::With => todo!(),
                TokenKind::Let => todo!(),
                TokenKind::Where => todo!(),
                TokenKind::Do => todo!(),
                TokenKind::Equal => todo!(),
                TokenKind::Colon => todo!(),
                TokenKind::LeftArrow => todo!(),
                TokenKind::At => todo!(),
                TokenKind::VerticalLine => todo!(),
            }
        }
        Ok(())
    }
}
