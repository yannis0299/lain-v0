use crate::utils::Position;

use eyre::{bail, Result};

#[derive(Clone)]
pub struct TokenStream {
    pub name: String,
    pub contents: String,
    pub pos: Position,
    pub idx: usize,
    pub stream: Vec<char>,
}

impl TokenStream {
    pub fn new(name: &str, contents: &str) -> Self {
        Self {
            name: String::from(name),
            contents: String::from(contents),
            pos: Position(1, 1),
            idx: 0,
            stream: contents.chars().collect(),
        }
    }

    pub fn advance(&mut self) -> Result<(Position, usize, char)> {
        let pos = self.pos;
        let (idx, c) = {
            if self.idx == self.stream.len() {
                bail!("TokenStream: Empty character stream");
            } else {
                let ret = (self.idx, self.stream[self.idx]);
                self.idx += 1;
                ret
            }
        };
        if c == '\t' || c == '\r' {
            bail!("TokenStream: Invalid whitespace escape character {:?}", c);
        } else if c == '\n' {
            self.pos.0 += 1;
            self.pos.1 = 1;
        } else {
            self.pos.1 += 1;
        }
        Ok((pos, idx, c))
    }
}
