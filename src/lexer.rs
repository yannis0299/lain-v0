use crate::{
    matcher::MonadMatcher,
    stream::TokenStream,
    utils::{Position, Span},
};

use eyre::bail;
use lazy_static::lazy_static;
use std::collections::HashSet;

pub fn predicate<P>(p: P) -> MonadMatcher<TokenStream, (Position, usize, char)>
where
    P: Fn(char) -> bool + 'static,
{
    MonadMatcher::new(move |state: &mut TokenStream| {
        let ret @ (_, _, c) = state.advance()?;
        if !p(c) {
            bail!("predicate: Failed to assert predicate on {:?}", c)
        }
        Ok(ret)
    })
}

pub fn alpha() -> MonadMatcher<TokenStream, (Position, usize, char)> {
    predicate(|c| c.is_alphabetic())
}

pub fn alphanum() -> MonadMatcher<TokenStream, (Position, usize, char)> {
    predicate(|c| c.is_alphanumeric())
}

pub fn digit() -> MonadMatcher<TokenStream, (Position, usize, char)> {
    predicate(|c| c.is_ascii_digit())
}

pub fn single(t: char) -> MonadMatcher<TokenStream, (Position, usize, char)> {
    predicate(move |c| c == t)
}

pub fn whitespace() -> MonadMatcher<TokenStream, (Position, usize, char)> {
    predicate(|c| c == ' ' || c == '\n')
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Match,
    With,
    If,
    Then,
    Else,
    Let,
    Where,
    Do,
    Backslash,
    RightFatArrow,
    Equal,
    Colon,
    LeftArrow,
    At,
    VerticalLine,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Underscore,
    Comma,
    Integer,
    Operator,
    Identifier,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: Position,
    pub span: Span,
}

pub fn integer() -> MonadMatcher<TokenStream, Token> {
    single('-')
        .optional()
        .chain(digit().many1())
        .map(|(sign, digits)| {
            let (pos, idx, mut len) = {
                if let Some((pos, idx, _)) = sign {
                    (pos, idx, 1usize)
                } else {
                    let (pos, idx, _) = digits[0];
                    (pos, idx, 0usize)
                }
            };
            len += digits.len();
            Token {
                kind: TokenKind::Integer,
                pos,
                span: Span(idx, idx + len),
            }
        })
}

pub fn keyword(name: &str, kind: TokenKind) -> MonadMatcher<TokenStream, Token> {
    let chars = name.chars().collect::<Vec<_>>();
    let mut m = MonadMatcher::pure(None);
    for c in chars {
        m = m
            .chain(single(c))
            .map(|(mpos, (pos, idx, _))| mpos.or(Some((pos, idx))));
    }
    let nm = String::from(name);
    m.then(move |mpos| match mpos {
        None => {
            let nm = nm.clone();
            MonadMatcher::failure(move || bail!("keyword: Did not match keyword {:?}", nm.clone()))
        }
        Some((pos, idx)) => MonadMatcher::pure(Token {
            kind,
            pos,
            span: Span(idx, idx + nm.len()),
        }),
    })
}

lazy_static! {
    static ref RESERVED_NAMES: HashSet<&'static str> =
        vec!["_", "match", "with", "if", "then", "else", "let", "where", "do"]
            .into_iter()
            .collect();
    static ref RESERVED_OPERATORS: HashSet<&'static str> =
        vec!["\\", "=>", "=", ":", "<-", "..", "@", "|"]
            .into_iter()
            .collect();
    static ref OPERATORS_LETTERS: Vec<char> = vec![
        ':', '!', '#', '$', '%', '&', '*', '+', '.', '/', '<', '=', '>', '@', '\\', '^', '|', '-',
        '~',
    ];
}

pub fn op_letter() -> MonadMatcher<TokenStream, (Position, usize, char)> {
    MonadMatcher::fold(
        OPERATORS_LETTERS
            .iter()
            .map(|&c| single(c))
            .collect::<Vec<_>>(),
    )
}

pub fn operator() -> MonadMatcher<TokenStream, Token> {
    let m = op_letter().many1();
    MonadMatcher::new(move |state: &mut TokenStream| {
        let letters = (m.0)(state)?;
        let (pos, idx, _) = letters[0];
        let len = letters.len();
        let repr = &state.contents[idx..idx + len];
        if RESERVED_OPERATORS.contains(repr) {
            bail!("operator: {:?} is a reserved operator name", repr)
        } else {
            Ok(Token {
                kind: TokenKind::Operator,
                pos,
                span: Span(idx, idx + len),
            })
        }
    })
}

pub fn identifier() -> MonadMatcher<TokenStream, Token> {
    let ident_start = single('_').or(alpha());
    let ident_letter = single('_').or(alphanum());
    let m = ident_start
        .chain(ident_letter.many())
        .map(|((pos, idx, _), u)| (pos, idx, 1 + u.len()));
    MonadMatcher::new(move |state: &mut TokenStream| {
        let (pos, idx, len) = (m.0)(state)?;
        let repr = &state.contents[idx..idx + len];
        if RESERVED_NAMES.contains(repr) {
            bail!("identifier: {:?} is a reserved identifier name", repr)
        } else {
            Ok(Token {
                kind: TokenKind::Identifier,
                pos,
                span: Span(idx, idx + len),
            })
        }
    })
}

pub fn lexeme() -> MonadMatcher<TokenStream, Token> {
    MonadMatcher::fold(vec![
        integer(),
        operator(),
        identifier(),
        keyword("(", TokenKind::LeftParen),
        keyword(")", TokenKind::RightParen),
        keyword("[", TokenKind::LeftBracket),
        keyword("]", TokenKind::RightBracket),
        keyword("_", TokenKind::Underscore),
        keyword(",", TokenKind::Comma),
        keyword("\\", TokenKind::Backslash),
        keyword("=>", TokenKind::RightFatArrow),
    ])
}

pub fn lexer() -> MonadMatcher<TokenStream, Vec<Token>> {
    (whitespace().many().chain(lexeme()).map(|(_, token)| token)).many()
}
