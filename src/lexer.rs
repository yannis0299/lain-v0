use crate::tokenizer::{Position, Span, Token, TokenKind, TokenParser, token_error};

pub fn single<'a>(c: char) -> TokenParser<'a, char> {
    TokenParser::new(move |tn| {
        let c1 = tn.advance()?;
        if c == c1 {
            Ok(c)
        } else {
            token_error!(
                tn,
                "single: Failed to match characters {:?} and {:?}",
                c,
                c1
            )
        }
    })
}

pub fn keyword<'a>(name: &'static str) -> TokenParser<'a, Token<'a>> {
    TokenParser::new(move |tn| {
        let start = tn.idx;
        let pos = tn.pos;
        for (idx, c) in name.char_indices() {
            let c1 = tn.advance()?;
            if c != c1 {
                return token_error!(
                    tn,
                    "keyword: Failed to match characters {:?} and {:?} at {}",
                    c,
                    c1,
                    idx
                );
            }
        }
        let len = name.len();
        Ok(Token {
            kind: TokenKind::Keyword,
            pos,
            span: Span(start, start + len),
            repr: &tn.contents[start..start + len],
        })
    })
}

pub fn alpha<'a>() -> TokenParser<'a, char> {
    TokenParser::new(move |tn| {
        let c = tn.advance()?;
        if c.is_alphabetic() {
            Ok(c)
        } else {
            token_error!(tn, "alpha: Not a valid alphabetical character {:?}", c)
        }
    })
}

pub fn alphanum<'a>() -> TokenParser<'a, char> {
    TokenParser::new(move |tn| {
        let c = tn.advance()?;
        if c.is_alphanumeric() {
            Ok(c)
        } else {
            token_error!(tn, "alphanum: Not a valid alphanumerical character {:?}", c)
        }
    })
}

pub fn identifier<'a>() -> TokenParser<'a, Token<'a>> {
    let p = single('_')
        .or(alpha())
        .chain(single('_').or(alphanum()).many())
        .map(|(_, t)| 1 + t.len());
    TokenParser::new(move |tn| {
        let pos = tn.pos;
        let idx = tn.idx;
        let len = (p.m)(tn)?;
        Ok(Token {
            kind: TokenKind::Identifier,
            pos,
            span: Span(idx, idx + len),
            repr: &tn.contents[idx..idx + len],
        })
    })
}

pub fn whitespaces<'a>() -> TokenParser<'a, Vec<Token<'a>>> {
    TokenParser::new(|tn| {
        // Eat trailing spaces before '\n' or non-space character
        if !tn.first_nonwhitespace {
            while let Some(&c) = tn.chars.peek()
                && c == ' '
            {
                tn.advance()?;
            }
        }
        // Here we either have a newline or a non-whitespace character
        let mut acc = vec![];
        if let Some(&c) = tn.chars.peek()
            && c != '\n'
        {
            Ok(acc)
        } else {
            // We have a newline
            tn.advance()?; // Eat newline
            loop {
                let idx = tn.idx;
                while let Some(&c) = tn.chars.peek()
                    && c == ' '
                // Eat heading spaces
                {
                    tn.advance()?;
                }
                // Same as above either we have a newline or a non-whitespace character
                if let Some(&c) = tn.chars.peek()
                    && c == '\n'
                {
                    acc.push(Token {
                        kind: TokenKind::Newline,
                        pos: Position(tn.pos.0, 1),
                        span: Span(idx, idx + tn.pos.1),
                        repr: &tn.contents[idx..idx + tn.pos.1],
                    });
                    tn.advance()?;
                    continue;
                } else {
                    if tn.pos.1 > 1 {
                        acc.push(Token {
                            kind: TokenKind::Indentation,
                            pos: Position(tn.pos.0, 1),
                            span: Span(idx, idx + tn.pos.1 - 1),
                            repr: &tn.contents[idx..idx + tn.pos.1 - 1],
                        });
                    }
                    return Ok(acc);
                }
            }
        }
    })
}

pub fn pfold<'a, T: 'a>(ps: Vec<TokenParser<'a, T>>) -> TokenParser<'a, T> {
    let mut ps = ps;
    let mut mp = ps.pop().expect("pfold: Empty parsers list");
    for p in ps {
        mp = mp.or(p);
    }
    mp
}

pub fn lexeme<'a>() -> TokenParser<'a, Token<'a>> {
    pfold(vec![
        keyword("\\"),
        keyword("=>"),
        keyword("="),
        keyword(":"),
        keyword("<-"),
        keyword("match"),
        keyword("if"),
        keyword("then"),
        keyword("else"),
        keyword("where"),
        keyword("do"),
        keyword("(").map(|tk| Token {
            kind: TokenKind::LeftParen,
            ..tk
        }),
        keyword(")").map(|tk| Token {
            kind: TokenKind::LeftParen,
            ..tk
        }),
        identifier(),
    ])
}

pub fn lexer<'a>() -> TokenParser<'a, Vec<Token<'a>>> {
    whitespaces()
        .chain(lexeme())
        .map(|(t, h)| {
            let mut t = t;
            t.push(h);
            t
        })
        .many()
        .map(|vs| vs.into_iter().flatten().collect::<Vec<_>>())
}
