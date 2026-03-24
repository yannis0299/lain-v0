use lazy_static::lazy_static;
use std::collections::HashSet;

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

pub fn digit<'a>() -> TokenParser<'a, char> {
    TokenParser::new(move |tn| {
        let c = tn.advance()?;
        if c.is_ascii_digit() {
            Ok(c)
        } else {
            token_error!(tn, "digit: Not a valid digital character {:?}", c)
        }
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

pub fn comment<'a>() -> TokenParser<'a, Token<'a>> {
    TokenParser::new(|tn| {
        let mut uv = [tn.advance()?, tn.advance()?];
        if uv == ['{', ':'] {
            loop {
                uv[0] = uv[1];
                uv[1] = tn.advance()?;
                if uv == [':', '}'] {
                    break;
                }
            }
            Ok(())
        } else {
            token_error!(
                tn,
                "comment: Comment blocks must start with {{: and and end with :}}"
            )
        }
    })
    .token(|_| TokenKind::Comment)
}

pub fn integer<'a>() -> TokenParser<'a, Token<'a>> {
    TokenParser::optional(single('-'))
        .chain(digit().many1())
        .map(|(s, t)| {
            if let Some(h) = s {
                let mut acc = vec![h];
                acc.extend(t);
                acc
            } else {
                t
            }
        })
        .token(|_| TokenKind::Integer)
}

pub fn character<'a>() -> TokenParser<'a, Token<'a>> {
    TokenParser::new(|tn| {
        if tn.advance()? == '\'' {
            let mut escaped = false;
            loop {
                let c = tn.advance()?;
                if !escaped && c == '\\' {
                    escaped = true;
                }
                if escaped {
                    escaped = false;
                }
                if !escaped && c == '\'' {
                    break;
                }
            }
            Ok(())
        } else {
            token_error!(
                tn,
                "character: Character literals must start with a single quote"
            )
        }
    })
    .token(|_| TokenKind::Character)
}

pub fn string<'a>() -> TokenParser<'a, Token<'a>> {
    TokenParser::new(|tn| {
        if tn.advance()? == '"' {
            let mut escaped = false;
            loop {
                let c = tn.advance()?;
                if !escaped && c == '\\' {
                    escaped = true;
                }
                if escaped {
                    escaped = false;
                }
                if !escaped && c == '"' {
                    break;
                }
            }
            Ok(())
        } else {
            token_error!(tn, "string: String literals must start with a double quote")
        }
    })
    .token(|_| TokenKind::String)
}

pub fn keyword<'a>(name: &'static str) -> TokenParser<'a, Token<'a>> {
    TokenParser::new(move |tn| {
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
        Ok(())
    })
    .token(|_| TokenKind::Keyword)
}

lazy_static! {
    static ref RESERVED_NAMES: HashSet<&'static str> = vec![
        "match", "with", "if", "then", "else", "let", "where", "do", "data", "type", "use"
    ]
    .into_iter()
    .collect();
    static ref RESERVED_OPERATORS: HashSet<&'static str> =
        vec!["\\", "=>", "=", ":", "<-", "..", "@", "|"]
            .into_iter()
            .collect();
}

pub fn op_letter<'a>() -> TokenParser<'a, char> {
    TokenParser::fold(
        vec![
            ':', '!', '#', '$', '%', '&', '*', '+', '.', '/', '<', '=', '>', '@', '\\', '^', '|',
            '-', '~',
        ]
        .into_iter()
        .map(single)
        .collect::<Vec<_>>(),
    )
}

pub fn operator<'a>() -> TokenParser<'a, Token<'a>> {
    op_letter().many1().token(|repr| {
        if RESERVED_OPERATORS.contains(repr) {
            TokenKind::Keyword
        } else {
            TokenKind::Operator
        }
    })
}

pub fn identifier<'a>() -> TokenParser<'a, Token<'a>> {
    let ident_start = TokenParser::or(single('_'), alpha());
    let ident_letter = TokenParser::or(single('_'), alphanum());
    ident_start
        .chain(ident_letter.many())
        .map(|(h, t)| {
            let mut acc = vec![h];
            acc.extend(t);
            acc
        })
        .token(|repr| {
            if RESERVED_NAMES.contains(repr) {
                TokenKind::Keyword
            } else {
                TokenKind::Identifier
            }
        })
}

pub fn lexeme<'a>() -> TokenParser<'a, Token<'a>> {
    TokenParser::fold(vec![
        comment(),
        integer(),
        character(),
        string(),
        operator(),
        identifier(),
        keyword("(").map(|tk| Token {
            kind: TokenKind::LeftParen,
            ..tk
        }),
        keyword(")").map(|tk| Token {
            kind: TokenKind::RightParen,
            ..tk
        }),
        keyword("[").map(|tk| Token {
            kind: TokenKind::LeftBracket,
            ..tk
        }),
        keyword("]").map(|tk| Token {
            kind: TokenKind::RightBracket,
            ..tk
        }),
        keyword("_").map(|tk| Token {
            kind: TokenKind::Underscore,
            ..tk
        }),
        keyword(",").map(|tk| Token {
            kind: TokenKind::Comma,
            ..tk
        }),
    ])
}

pub fn token_or_block<'a>() -> TokenParser<'a, Vec<Token<'a>>> {
    whitespaces().chain(lexeme()).map(|(t, h)| {
        let mut t = t;
        t.push(h);
        t
    })
}

pub fn lexer<'a>() -> TokenParser<'a, Vec<Token<'a>>> {
    token_or_block()
        .many()
        .map(|vs| vs.into_iter().flatten().collect::<Vec<_>>())
}
