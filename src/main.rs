#![allow(unused, dead_code, unused_imports)]
mod tokenizer;
mod tu;

use fallible_iterator::{FallibleIterator, IntoFallibleIterator};

use crate::{tokenizer::Tokenizer, tu::TU};

fn main() {
    let expr = "\\ xyz_fdhdfjk _ _x _0 x0 xA => ()";
    let tu = TU {
        filename: String::from("foo.ln"),
        contents: String::from(expr),
    };
    let mut token_stream = Tokenizer::new(&tu).into_fallible_iter();
    println!("Expr = {expr:?}");
    loop {
        match token_stream.next() {
            Err(err) => panic!("TokenizerError: {err:?}"),
            Ok(None) => break,
            Ok(Some(token)) => {
                println!("{token:?}")
            }
        }
    }
}
