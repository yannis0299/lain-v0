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
    Tuple(AST, ASTs, AST), // (e1, .., eN) at least 0 elements
    List(ASTs),            // [e1, .., eN] can have 0 elements
    // Composite expression
    Lambda(AST, AST),            // \ e1 => e2
    Application(AST, ASTs, AST), // e1 .. eN
    IfThenElse(AST, AST, AST),   // if e1 then e2 else e3
}

#[allow(clippy::upper_case_acronyms)]
pub type AST = Box<RawAST>;
pub type ASTs = Vec<RawAST>;

#[derive(Debug, Clone)]
pub enum Frame {
    AtomsAcc(ASTs),
    TupleAcc(ASTs),
    ListAcc(ASTs),
    IfWaitingForThen,
    IfThenWaitingForElse(AST),
    IfThenElseWaitingForReduction(AST, AST),
    LambdaWaitingForFatArrow,
    LambdaWaitingForReduction(AST),
    ReducedAST(RawAST),
}

pub struct ASTBuilder<'a> {
    pub contents: &'a str,
    pub frame_stack: Vec<Frame>,
}

impl<'a> ASTBuilder<'a> {
    pub fn build(mut self, tokens: &[Token]) -> Result<AST> {
        self.eat_tokens(tokens)?;
        match self.frame_stack.pop() {
            Some(Frame::ReducedAST(ast)) => Ok(Box::new(ast)),
            _ => bail!("ASTBuilder: Malformed expression"),
        }
    }

    pub fn eat_tokens(&mut self, tokens: &[Token]) -> Result<()> {
        for token in tokens {
            match token.kind {
                TokenKind::Underscore => {
                    self.push_atom(RawAST::Wildcard);
                }
                TokenKind::Integer => {
                    let repr = &self.contents[token.span.0..token.span.1];
                    let value = repr.parse()?;
                    self.push_atom(RawAST::Integer(value));
                }
                TokenKind::Identifier => {
                    let repr = &self.contents[token.span.0..token.span.1];
                    let value = String::from(repr);
                    self.push_atom(RawAST::Variable(value));
                }
                TokenKind::LeftParen => {
                    let frame = Frame::TupleAcc(vec![]);
                    self.push_frame(frame);
                }
                TokenKind::LeftBracket => {
                    let frame = Frame::ListAcc(vec![]);
                    self.push_frame(frame);
                }
                TokenKind::Comma => {
                    self.reduce_frames_until(|frame| {
                        matches!(frame, Frame::TupleAcc(_) | Frame::ListAcc(_))
                    })?;
                }
                TokenKind::RightParen => {
                    self.reduce_frames_until(|frame| matches!(frame, Frame::TupleAcc(_)))?;
                    match self.pop_frame()? {
                        Frame::TupleAcc(mut acc) => {
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
                            self.push_atom(atom);
                        }
                        _ => bail!(
                            "ASTBuilder: Attempting to close tuple without an opening parenthesis"
                        ),
                    }
                }
                TokenKind::RightBracket => {
                    self.reduce_frames_until(|frame| matches!(frame, Frame::ListAcc(_)))?;
                    match self.pop_frame()? {
                        Frame::ListAcc(acc) => {
                            let atom = RawAST::List(acc);
                            self.push_atom(atom);
                        }
                        _ => {
                            bail!("ASTBuilder: Attempting to close list without an opening bracket")
                        }
                    }
                }
                TokenKind::If => {
                    let frame = Frame::IfWaitingForThen;
                    self.push_frame(frame);
                }
                TokenKind::Then => {
                    self.reduce_frames_until(|frame| {
                        matches!(frame, Frame::IfThenWaitingForElse(_))
                    })?;
                }
                TokenKind::Else => {
                    self.reduce_frames_until(|frame| {
                        matches!(frame, Frame::IfThenElseWaitingForReduction(_, _))
                    })?;
                }
                TokenKind::Backslash => {
                    let frame = Frame::LambdaWaitingForFatArrow;
                    self.push_frame(frame);
                }
                TokenKind::RightFatArrow => {
                    self.reduce_frames_until(|frame| {
                        matches!(frame, Frame::LambdaWaitingForReduction(_))
                    })?;
                }
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
        self.reduce_frames_until(|frame| matches!(frame, Frame::ReducedAST(_)))
    }

    pub fn last_frame(&self) -> Result<&Frame> {
        self.frame_stack
            .last()
            .wrap_err("ASTBuilder: attempting to reference last frame in an empty stack")
    }

    pub fn new(contents: &'a str) -> Self {
        Self {
            contents,
            frame_stack: vec![],
        }
    }

    pub fn pop_frame(&mut self) -> Result<Frame> {
        self.frame_stack
            .pop()
            .wrap_err("ASTBuilder: attempting to pop last frame in an empty stack")
    }

    pub fn push_atom(&mut self, atom: RawAST) {
        match self.frame_stack.last_mut() {
            Some(Frame::AtomsAcc(acc)) => acc.push(atom),
            _ => self.push_frame(Frame::AtomsAcc(vec![atom])),
        };
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.frame_stack.push(frame);
    }

    pub fn reduce_frames_until<P>(&mut self, pred: P) -> Result<()>
    where
        P: Fn(&Frame) -> bool + 'static,
    {
        loop {
            self.reduce_top_frame()?;
            if self.frame_stack.is_empty() || (pred)(self.last_frame()?) {
                break;
            }
        }
        Ok(())
    }

    pub fn reduce_top_frame(&mut self) -> Result<()> {
        match self.pop_frame()? {
            Frame::AtomsAcc(mut acc) => {
                let ast = match acc.len() {
                    0 => RawAST::Empty,
                    1 => acc.pop().wrap_err("unreachable")?,
                    _ => {
                        let mut tail = acc.split_off(1);
                        let head = acc.pop().wrap_err("unreachable")?;
                        let last = tail.pop().wrap_err("unreachable")?;
                        RawAST::Application(Box::new(head), tail, Box::new(last))
                    }
                };
                self.push_frame(Frame::ReducedAST(ast));
                Ok(())
            }
            Frame::TupleAcc(mut acc) => {
                acc.push(RawAST::Empty);
                self.push_frame(Frame::TupleAcc(acc));
                Ok(())
            }
            Frame::ListAcc(mut acc) => {
                acc.push(RawAST::Empty);
                self.push_frame(Frame::TupleAcc(acc));
                Ok(())
            }
            Frame::IfWaitingForThen => {
                bail!("ASTBuilder: Empty expression between if and then tokens")
            }
            Frame::IfThenWaitingForElse(_) => {
                bail!("ASTBuilder: Empty expression between then and else tokens")
            }
            Frame::IfThenElseWaitingForReduction(_, _) => {
                bail!("ASTBuilder: Empty expression after else token")
            }
            Frame::LambdaWaitingForFatArrow => {
                bail!("ASTBuilder: Empty expression backslash and fat arrow")
            }
            Frame::LambdaWaitingForReduction(_) => {
                bail!("ASTBuilder: Empty lambda body after fat arrow")
            }
            Frame::ReducedAST(ast) => {
                if self.frame_stack.is_empty() {
                    self.push_frame(Frame::ReducedAST(ast));
                    Ok(())
                } else {
                    match self.pop_frame()? {
                        Frame::AtomsAcc(_) => bail!("ASTBuilder: Ambiguis expression after an atom list, please use parenthesis"),
                        Frame::TupleAcc(mut acc) => {
                            acc.push(ast);
                            self.push_frame(Frame::TupleAcc(acc));
                            Ok(())
                        },
                        Frame::ListAcc(mut acc) =>  {
                            acc.push(ast);
                            self.push_frame(Frame::ListAcc(acc));
                            Ok(())
                        },
                        Frame::IfWaitingForThen => {
                            let frame = Frame::IfThenWaitingForElse(Box::new(ast));
                            self.push_frame(frame);
                            Ok(())
                        },
                        Frame::IfThenWaitingForElse(e1) => {
                            let frame = Frame::IfThenElseWaitingForReduction(e1, Box::new(ast));
                            self.push_frame(frame);
                            Ok(())
                        },
                        Frame::IfThenElseWaitingForReduction(e1, e2) => {
                            let e3 = Box::new(ast);
                            let ast = RawAST::IfThenElse(e1, e2, e3);
                            self.push_frame(Frame::ReducedAST(ast));
                            Ok(())
                        },
                        Frame::LambdaWaitingForFatArrow => {
                            let frame = Frame::LambdaWaitingForReduction(Box::new(ast));
                            self.push_frame(frame);
                            Ok(())
                        },
                        Frame::LambdaWaitingForReduction(e1) => {
                            let e2 = Box::new(ast);
                            let ast = RawAST::Lambda(e1, e2);
                            self.push_frame(Frame::ReducedAST(ast));
                            Ok(())
                        },
                        Frame::ReducedAST(_) => bail!("ASTBuilder: Ambiguis expression after another expression, please use parenthesis"),
                    }
                }
            }
        }
    }
}
