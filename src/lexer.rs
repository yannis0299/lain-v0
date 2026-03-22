use std::{cell::RefCell, str::Chars};

use crate::monadic::{ParseError, Parser};

#[derive(Debug, Clone, Copy)]
pub struct Position(pub usize, pub usize);

#[derive(Debug, Clone, Copy)]
pub struct Span(pub usize, pub usize);

#[derive(Debug, Clone, Copy)]
pub enum TokenKind {
    Backslash,
    RightArrow,
    LeftParen,
    RightParen,
    Identifier,
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub pos: Position,
    pub span: Span,
    pub repr: &'a str,
}

#[derive(Clone)]
pub struct Tokenizer<'a> {
    name: &'a str,
    contents: &'a str,
    chars: Chars<'a>,
    pos: Position,
    idx: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn advance(&'a mut self) -> Result<char, ParseError<Tokenizer<'a>>>
}

pub fn advance<'a>() -> Parser<Tokenizer<'_a>, char> {
    Parser::unit(Box::new(|state| {
        Ok(('a', state))
        // let mut saved_state = state.clone();
        // let mut state = state;
        // RefCell
        // match state.chars.next() {
        //     Some('\n') => {
        //         state.pos.0 += 1;
        //         state.pos.1 = 1;
        //         state.idx += 1;
        //         Ok(('\n', state))
        //     }
        //     Some(c) => {
        //         state.pos.1 += 1;
        //         state.idx += 1;
        //         Ok((c, state))
        //     }
        //     None => Err(ParseError {
        //         state: saved_state,
        //         msg: format!("Unexpected end of stream"),
        //     }),
        // }
    }))

    // self.chars.next().map(|elem @ (_, char)| {
    //         match char {
    //             '\n' => {
    //                 // Advance line
    //                 self.pos.0 += 1; // line += 1
    //                 self.pos.1 = 1; // column = 1
    //             }
    //             _ => self.pos.1 += 1, // Advance column
    //         }
    //         elem
    //     })
}

pub fn keyword<'a>(pattern: &'static str) -> Parser<Tokenizer<'a>, Token<'a>> {
    todo!()
}
