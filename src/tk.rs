use fallible_iterator::FallibleIterator;

use std::{iter::Peekable, path::Path, str::CharIndices};

use crate::tu::TU;

#[derive(Debug, Clone, Copy)]
pub struct Position(pub usize, pub usize);

impl Position {
    #[inline]
    pub fn line(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn column(&self) -> usize {
        self.1
    }
}

impl Default for Position {
    fn default() -> Self {
        Self(1usize, 0usize)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Span(pub usize, pub usize);

impl Span {
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Self(start, end)
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.1
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.1 - self.0 + 1
    }
}

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

pub struct Tokenizer<'a> {
    filename: &'a Path,
    contents: &'a str,
    chars: Peekable<CharIndices<'a>>,
    pos: Position,
}

#[derive(Debug, Clone)]
pub struct TokenizerError {
    pub message: String,
    pub filename: String,
    pub pos: Position,
}

impl<'a> Tokenizer<'a> {
    pub fn new(tu: &'a TU) -> Self {
        Tokenizer {
            filename: Path::new(tu.filename.as_str()),
            pos: Position::default(),
            contents: tu.contents.as_str(),
            chars: tu.contents.char_indices().peekable(),
        }
    }

    fn advance_char_by_one(&mut self) -> Option<(usize, char)> {
        self.chars.next().map(|elem @ (_, char)| {
            match char {
                '\n' => {
                    // Advance line
                    self.pos.0 += 1; // line += 1
                    self.pos.1 = 1; // column = 1
                }
                _ => self.pos.1 += 1, // Advance column
            }
            elem
        })
    }

    fn advance_char(&mut self) -> Option<(usize, char)> {
        loop {
            let next = self.advance_char_by_one();
            match next {
                Some((_, ' ')) | Some((_, '\n')) => continue,
                _ => break next,
            }
        }
    }

    fn error(&self, message: String) -> TokenizerError {
        let filename = {
            self.filename
                .file_name()
                .and_then(|os_str| os_str.to_str())
                .map(String::from)
                .unwrap_or(String::from("<unknown>"))
        };
        TokenizerError {
            filename,
            pos: self.pos,
            message,
        }
    }
}

impl<'a> FallibleIterator for Tokenizer<'a> {
    type Item = Token<'a>;
    type Error = TokenizerError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        match self.advance_char() {
            None => Ok(None),
            Some((idx, '\\')) => Ok(Some(Token {
                kind: TokenKind::Backslash,
                pos: self.pos,
                span: Span::new(idx, idx + 1),
                repr: &self.contents[idx..idx + 1],
            })),
            Some((idx, '(')) => Ok(Some(Token {
                kind: TokenKind::LeftParen,
                pos: self.pos,
                span: Span::new(idx, idx + 1),
                repr: &self.contents[idx..idx + 1],
            })),
            Some((idx, ')')) => Ok(Some(Token {
                kind: TokenKind::RightParen,
                pos: self.pos,
                span: Span::new(idx, idx + 1),
                repr: &self.contents[idx..idx + 1],
            })),
            Some((idx, '=')) => {
                let pos = self.pos;
                match self.advance_char() {
                    Some((_, '>')) => Ok(Some(Token {
                        kind: TokenKind::RightParen,
                        pos,
                        span: Span::new(idx, idx + 2),
                        repr: &self.contents[idx..idx + 2],
                    })),
                    Some(char) => Err(self.error(format!(
                        "Unexpected token while trying to match right arrow: {:?}",
                        char
                    ))),
                    None => Err(self.error(String::from(
                        "Unexpected end of file while trying to match right arrow",
                    ))),
                }
            }
            Some((idx, char)) => {
                if char.is_alphabetic() || char == '_' {
                    let pos = self.pos;
                    let mut len = 1;
                    while let Some(true) = self
                        .chars
                        .peek()
                        .map(|(_, peeked)| peeked.is_alphanumeric() || *peeked == '_')
                    {
                        self.chars.next();
                        self.pos.1 += 1;
                        len += 1;
                    }
                    Ok(Some(Token {
                        kind: TokenKind::Identifier,
                        pos,
                        span: Span::new(idx, idx + len),
                        repr: &self.contents[idx..idx + len],
                    }))
                } else {
                    Err(self.error(format!("Unexpected token {:?}", char)))
                }
            }
        }
    }
}
