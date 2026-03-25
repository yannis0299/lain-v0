mod ast;
mod lexer;
mod matcher;
mod stream;
mod utils;

use crate::lexer::lexer;
use crate::stream::TokenStream;

use ast::ASTBuilder;
use eyre::Result;
use std::fs::File;
use std::io::Read;

fn main() -> Result<()> {
    let mut file = File::open("test.ln")?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    println!("-- File: ---------------------");
    print!("{:#}", buf);
    println!("-- Tokens: -------------------");
    let mut token_stream = TokenStream::new("test.ln", &buf[..]);
    let token_lexer = lexer();
    let tokens = (token_lexer.0)(&mut token_stream)?;
    for token in &tokens {
        let unprintable = format!("{:?}", token);
        println!(
            "{:^16} | {}",
            format!("{:?}", &buf[token.span.0..token.span.1]),
            unprintable
        );
    }
    println!("-- AST: ----------------------");
    let mut ast_builder = ASTBuilder::new(&buf);
    ast_builder.eat_tokens(&tokens[..])?;
    println!("------------------------------");
    Ok(())
}
