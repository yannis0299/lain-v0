use crate::tokenizer::{Position, Span, Token, TokenKind, TokenParser};

pub fn single(c: char) -> TokenParser<char> {
    TokenParser::new(move |tn| {
        let c1 = tn.advance()?;
        if c == c1 {
            Ok(c)
        } else {
            Err(tn.error(format!(
                "single: Failed to match characters {:?} and {:?}",
                c, c1
            )))
        }
    })
}

pub fn keyword(name: &'static str) -> TokenParser<(Position, Span)> {
    TokenParser::new(move |tn| {
        let start = tn.idx;
        let pos = tn.pos;
        for (idx, c) in name.char_indices() {
            let c1 = tn.advance()?;
            if c != c1 {
                return Err(tn.error(format!(
                    "keyword: Failed to match characters {:?} and {:?} at {}",
                    c, c1, idx
                )));
            }
        }
        Ok((pos, Span(start, start + name.len())))
    })
}

pub fn alpha() -> TokenParser<char> {
    TokenParser::new(move |tn| {
        let c = tn.advance()?;
        if c.is_alphabetic() {
            Ok(c)
        } else {
            Err(tn.error(format!("alpha: Not a valid alphabetical character {:?}", c)))
        }
    })
}

pub fn alphanum() -> TokenParser<char> {
    TokenParser::new(move |tn| {
        let c = tn.advance()?;
        if c.is_alphanumeric() {
            Ok(c)
        } else {
            Err(tn.error(format!(
                "alphanum: Not a valid alphanumerical character {:?}",
                c
            )))
        }
    })
}

pub fn identifier() -> TokenParser<Token> {
    let p = single('_')
        .or(alpha())
        .chain(single('_').or(alphanum()).many())
        .map(|(_, t)| 1 + t.len());
    TokenParser::new(move |tn| {
        let pos = tn.pos;
        let idx = tn.idx;
        let len = (p.m)(tn)?;
        Ok(Token {
            pos,
            span: Span(idx, idx + len),
            kind: TokenKind::Identifier,
        })
    })
}

pub fn whitespaces() -> TokenParser<Vec<char>> {
    single(' ').or(single('\t')).or(single('\n')).many()
}

pub fn skip<T: 'static>(p: TokenParser<T>) -> TokenParser<()> {
    p.optional().map(|_| ())
}

pub fn lexer() -> TokenParser<Vec<Token>> {
    skip(whitespaces())
        .then(|_| {
            keyword("\\")
                .map(|(pos, span)| Token {
                    kind: TokenKind::Backslash,
                    pos,
                    span,
                })
                .or(keyword("=>").map(|(pos, span)| Token {
                    kind: TokenKind::RightArrow,
                    pos,
                    span,
                }))
                .or(keyword("(").map(|(pos, span)| Token {
                    kind: TokenKind::LeftParen,
                    pos,
                    span,
                }))
                .or(keyword(")").map(|(pos, span)| Token {
                    kind: TokenKind::RightParen,
                    pos,
                    span,
                }))
                .or(identifier())
        })
        .many()
}
