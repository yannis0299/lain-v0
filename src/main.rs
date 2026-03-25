mod ast;
mod lexer;
mod matcher;
mod stream;
mod utils;

use crate::lexer::lexer;
use crate::stream::TokenStream;

use ast::ASTBuilder;
use eyre::{ContextCompat, Result};
use std::env::args;
use std::fs::File;
use std::io::Read;

fn do_file(filename: &str) -> Result<()> {
    let mut file = File::open(filename)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    println!("-- File: {:-<21}", format!("{} ", filename));
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
    let ast_builder = ASTBuilder::new(&buf);
    let ast = ast_builder.build(&tokens)?;
    println!("AST = {:#?}", ast);
    println!("------------------------------");
    Ok(())
}

fn main() -> Result<()> {
    let mut argv = args();
    argv.next().wrap_err("unreachable")?;
    for filename in argv {
        do_file(&filename)?;
    }
    Ok(())
}
