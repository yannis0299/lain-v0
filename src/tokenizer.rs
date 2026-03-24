use crate::translation_unit::TU;

use std::error::Error;
use std::fmt::Display;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, Copy)]
pub struct Position(pub usize, pub usize);

#[derive(Debug, Clone, Copy)]
pub struct Span(pub usize, pub usize);

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Underscore,
    Comma,
    Indentation,
    Comment,
    Newline,
    Keyword,
    Integer,
    Character,
    String,
    Operator,
    Identifier,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub pos: Position,
    pub span: Span,
    pub repr: &'a str,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct TokenError {
    pub src: String,
    pub pos: Position,
    pub idx: usize,
    pub msg: String,
}

impl Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl Error for TokenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        self.msg.as_str()
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }

    fn provide<'a>(&'a self, _request: &mut std::error::Request<'a>) {}
}

#[derive(Clone)]
pub struct Tokenizer<'a> {
    pub name: &'a str,
    pub contents: &'a str,
    pub pos: Position,
    pub first_nonwhitespace: bool,
    pub idx: usize,
    pub chars: Peekable<Chars<'a>>,
}

macro_rules! token_error {
    // Base case:
    ($x:expr) => ($x);
    // `$x` followed by at least one `$y,`
    ($x:expr, $($y:expr),+) => (
        // Call `find_min!` on the tail `$y`
        Err(($x).error(format!($($y),+)))
    )
}
pub(crate) use token_error;

impl<'a> Tokenizer<'a> {
    pub fn new(tu: &'a TU) -> Tokenizer<'a> {
        Tokenizer {
            name: tu.filename.as_str(),
            contents: tu.contents.as_str(),
            pos: Position(1, 1),
            first_nonwhitespace: true,
            idx: 0,
            chars: tu.contents.chars().peekable(),
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
                self.first_nonwhitespace = true;
                Ok('\n')
            }
            Some(c @ '\t') | Some(c @ '\r') => {
                token_error!(
                    self,
                    "advance: Invalid whitespace character {:?}\nSpaces and newlines are the only supported whitespaces characters!",
                    c
                )
            }
            Some(c) => {
                self.pos.1 += 1;
                self.idx += 1;
                if c != ' ' {
                    self.first_nonwhitespace = false;
                }
                Ok(c)
            }
            None => token_error!(self, "Unexpected end of character stream"),
        }
    }
}

pub type TokenMatcher<'a, T> = dyn Fn(&mut Tokenizer<'a>) -> Result<T, TokenError> + 'a;

pub struct TokenParser<'a, T> {
    pub m: Box<TokenMatcher<'a, T>>,
}

#[allow(unused)]
impl<'a, T> TokenParser<'a, T>
where
    T: 'a,
{
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut Tokenizer<'a>) -> Result<T, TokenError> + 'a,
    {
        TokenParser { m: Box::new(f) }
    }

    pub fn run(self, mut tn: Tokenizer<'a>) -> Result<T, TokenError> {
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

    pub fn map<U, F>(self, f: F) -> TokenParser<'a, U>
    where
        U: 'a,
        F: Fn(T) -> U + 'a,
    {
        TokenParser::new(move |tn| {
            let x = (self.m)(tn)?;
            Ok(f(x))
        })
    }

    pub fn then<U, F>(self, f: F) -> TokenParser<'a, U>
    where
        U: 'a,
        F: Fn(T) -> TokenParser<'a, U> + 'a,
    {
        TokenParser::new(move |tn| {
            let x = (self.m)(tn)?;
            let g = f(x);
            let y = (g.m)(tn)?;
            Ok(y)
        })
    }

    pub fn optional(self) -> TokenParser<'a, Option<T>> {
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

    pub fn or(self, that: TokenParser<'a, T>) -> TokenParser<'a, T> {
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

    pub fn chain<U>(self, that: TokenParser<'a, U>) -> TokenParser<'a, (T, U)>
    where
        U: 'a,
    {
        TokenParser::new(move |tn| {
            let x = (self.m)(tn)?;
            let y = (that.m)(tn)?;
            Ok((x, y))
        })
    }

    pub fn many(self) -> TokenParser<'a, Vec<T>> {
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

    pub fn many1(self) -> TokenParser<'a, Vec<T>> {
        TokenParser::new(move |tn| {
            let mut acc = Vec::new();
            loop {
                let saved = tn.clone();
                match (self.m)(tn) {
                    Err(_) => {
                        *tn = saved;
                        if acc.is_empty() {
                            return token_error!(tn, "many1: Expecting as least one match");
                        } else {
                            return Ok(acc);
                        }
                    }
                    Ok(x) => {
                        acc.push(x);
                    }
                }
            }
        })
    }

    pub fn fold(ps: Vec<TokenParser<'a, T>>) -> TokenParser<'a, T> {
        let mut ps = ps;
        let mut mp = ps.pop().expect("pfold: Empty parsers list");
        for p in ps {
            mp = mp.or(p);
        }
        mp
    }

    pub fn token<F>(self, f: F) -> TokenParser<'a, Token<'a>>
    where
        F: Fn(&str) -> TokenKind + 'a,
    {
        TokenParser::new(move |tn| {
            let pos = tn.pos;
            let idx = tn.idx;
            let _ = (self.m)(tn)?;
            let len = tn.idx - idx;
            let span = Span(idx, idx + len);
            let repr = &tn.contents[span.0..span.1];
            let kind = f(repr);
            Ok(Token {
                kind,
                pos,
                span,
                repr,
            })
        })
    }
}
