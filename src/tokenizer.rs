use crate::translation_unit::TU;

use std::str::Chars;

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
pub struct Token {
    pub kind: TokenKind,
    pub pos: Position,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TokenError {
    pub src: String,
    pub pos: Position,
    pub idx: usize,
    pub msg: String,
}

#[derive(Clone)]
pub struct Tokenizer<'a> {
    pub name: &'a str,
    pub contents: &'a str,
    pub pos: Position,
    pub idx: usize,
    pub chars: Chars<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(tu: &'a TU) -> Tokenizer<'a> {
        Tokenizer {
            name: tu.filename.as_str(),
            contents: tu.contents.as_str(),
            pos: Position(1, 1),
            idx: 0,
            chars: tu.contents.chars(),
        }
    }

    pub fn error<S>(&self, msg: S) -> TokenError
    where
        S: Into<String> + 'static,
    {
        TokenError {
            src: String::from(self.name),
            pos: self.pos,
            idx: self.idx,
            msg: msg.into(),
        }
    }

    pub fn advance(&mut self) -> Result<char, TokenError> {
        match self.chars.next() {
            Some('\n') => {
                self.pos.0 += 1;
                self.pos.1 = 1;
                self.idx += 1;
                Ok('\n')
            }
            Some(c) => {
                self.pos.1 += 1;
                self.idx += 1;
                Ok(c)
            }
            None => Err(self.error("Unexpected end of character stream")),
        }
    }
}

pub type TokenMatcher<T> = dyn Fn(&mut Tokenizer<'_>) -> Result<T, TokenError> + 'static;

pub struct TokenParser<T> {
    pub m: Box<TokenMatcher<T>>,
}

impl<T> TokenParser<T>
where
    T: 'static,
{
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut Tokenizer<'_>) -> Result<T, TokenError> + 'static,
    {
        TokenParser { m: Box::new(f) }
    }

    pub fn run(self, mut tn: Tokenizer<'_>) -> Result<T, TokenError> {
        (self.m)(&mut tn)
    }

    pub fn pure(ret: T) -> Self
    where
        T: Copy,
    {
        Self::new(move |_| Ok(ret))
    }

    pub fn failure<S>(msg: S) -> Self
    where
        S: Into<String> + 'static,
    {
        let msg = msg.into();
        Self::new(move |tn| {
            Err(TokenError {
                src: String::from(tn.name),
                pos: tn.pos,
                idx: tn.idx,
                msg: msg.clone(),
            })
        })
    }

    pub fn map<U, F>(self, f: F) -> TokenParser<U>
    where
        U: 'static,
        F: Fn(T) -> U + 'static,
    {
        TokenParser::new(move |tn| {
            let x = (self.m)(tn)?;
            Ok(f(x))
        })
    }

    pub fn then<U, F>(self, f: F) -> TokenParser<U>
    where
        U: 'static,
        F: Fn(T) -> TokenParser<U> + 'static,
    {
        TokenParser::new(move |tn| {
            let x = (self.m)(tn)?;
            let g = f(x);
            let y = (g.m)(tn)?;
            Ok(y)
        })
    }

    pub fn optional(self) -> TokenParser<Option<T>> {
        TokenParser::new(move |tn| {
            let saved = tn.clone();
            match (self.m)(tn) {
                Err(_) => {
                    *tn = saved;
                    Ok(None)
                }
                Ok(x) => Ok(Some(x)),
            }
        })
    }

    pub fn or(self, that: TokenParser<T>) -> TokenParser<T> {
        TokenParser::new(move |tn| {
            let saved = tn.clone();
            match (self.m)(tn) {
                Err(_) => {
                    *tn = saved;
                    (that.m)(tn)
                }
                Ok(x) => Ok(x),
            }
        })
    }

    pub fn chain<U>(self, that: TokenParser<U>) -> TokenParser<(T, U)>
    where
        U: 'static,
    {
        TokenParser::new(move |tn| {
            let x = (self.m)(tn)?;
            let y = (that.m)(tn)?;
            Ok((x, y))
        })
    }

    pub fn many(self) -> TokenParser<Vec<T>> {
        TokenParser::new(move |tn| {
            let mut acc = Vec::new();
            loop {
                let saved = tn.clone();
                match (self.m)(tn) {
                    Err(_) => {
                        *tn = saved;
                        return Ok(acc);
                    }
                    Ok(x) => {
                        acc.push(x);
                    }
                }
            }
        })
    }
}

pub struct TokenStream<'a, T> {
    tn: Tokenizer<'a>,
    p: TokenParser<T>,
}

impl<'a, T> TokenStream<'a, T>
where
    T: 'static,
{
    pub fn new(tu: &'a TU, p: TokenParser<T>) -> Self {
        Self {
            tn: Tokenizer::new(tu),
            p,
        }
    }

    pub fn next(&mut self) -> Result<T, TokenError> {
        (self.p.m)(&mut self.tn)
    }
}
