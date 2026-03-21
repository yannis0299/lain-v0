use fallible_iterator::{FallibleIterator, Peekable};

use crate::{
    tk::{Position, Span, Token, TokenKind, Tokenizer, TokenizerError},
    tu::TU,
};

#[derive(Debug, Clone)]
pub enum ASTKind<'a> {
    Variable(Token<'a>),
    Lambda(Token<'a>, Vec<Token<'a>>, Box<AST<'a>>),
    Application(Box<AST<'a>>, Vec<AST<'a>>, Box<AST<'a>>),
}

#[derive(Debug, Clone)]
pub struct AST<'a> {
    pos: Position,
    span: Span,
    repr: &'a str,
    kind: ASTKind<'a>,
}

impl<'a> AST<'a> {
    pub fn variable(token: Token<'a>) -> Self {
        Self {
            pos: token.pos,
            span: token.span,
            repr: token.repr,
            kind: ASTKind::Variable(token),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ASTError {
    pub message: String,
    pub filename: String,
    pub pos: Position,
}

pub struct ASTBuilder<'a> {
    tu: &'a TU,
    tokenizer: Peekable<Tokenizer<'a>>,
    pos: Position,
}

impl<'a> ASTBuilder<'a> {
    pub fn new(tu: &'a TU) -> Self {
        Self {
            tu,
            tokenizer: Tokenizer::new(tu).peekable(),
            pos: Position(1usize, 1usize),
        }
    }

    fn tokenizer_error(err: TokenizerError) -> ASTError {
        ASTError {
            message: err.message,
            filename: err.filename,
            pos: err.pos,
        }
    }

    fn error(&self, message: String) -> ASTError {
        let filename = self.tu.filename.clone();
        ASTError {
            message,
            filename,
            pos: self.pos,
        }
    }

    fn advance_token(&mut self) -> Result<Token<'a>, ASTError> {
        match self.tokenizer.next() {
            Err(err) => Err(Self::tokenizer_error(err)),
            Ok(None) => Err(self.error(String::from("Unexpected end of expresion"))),
            Ok(Some(token)) => {
                self.pos = token.pos;
                Ok(token)
            }
        }
    }

    pub fn build(mut self) -> Result<AST<'a>, ASTError> {
        match self.advance_token() {
            Err(err) => Err(err),
            Ok(token) => match token.kind {
                TokenKind::Backslash => match self.advance_token() {
                    Err(err) => Err(err),
                    Ok(token) => match token.kind {
                        TokenKind::Identifier => {
                            let head = token;
                            let tail = Vec::new();
                            todo!()
                        }
                        _ => Err(self.error(format!(
                            "Unexpected token {:?}, was expecting identifier",
                            token
                        ))),
                    },
                },
                TokenKind::RightArrow => {
                    Err(self.error(String::from("Unexpected token RIGHT_ARROW")))
                }
                TokenKind::LeftParen => todo!(),
                TokenKind::RightParen => todo!(),
                TokenKind::Identifier => Ok(AST::variable(token)),
            },
        }
    }
}
