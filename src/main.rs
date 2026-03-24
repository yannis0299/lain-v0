#![feature(error_generic_member_access)]
mod ast;
mod lexer;
mod tokenizer;
mod translation_unit;

use ast::ASTBuilder;
use eyre::Context;
use lexer::lexer;
use tokenizer::{TokenKind, Tokenizer};
use translation_unit::TU;

fn main() -> eyre::Result<()> {
    let tu = TU::from_file("test.ln").wrap_err("Could not read source file")?;
    let tn = Tokenizer::new(&tu);
    println!("-- File: ---------------------");
    print!("{:#}", &tn.contents);
    println!("-- Tokens: -------------------");
    let tokens = lexer().run(tn).wrap_err("Could not lex source file")?;
    for token in tokens
        .iter()
        .filter(|token| !token.kind.eq(&TokenKind::Comment))
    {
        let unprintable = format!("{:?}", token);
        println!(
            "{:^16} | {}",
            format!("{:?}", &tu.contents[token.span.0..token.span.1]),
            unprintable
        );
    }
    println!("-- AST: ----------------------");
    let mut ast_builder = ASTBuilder::new(tokens);
    ast_builder.exhaust()?;
    println!("frame_stack = {:#?}", ast_builder.frame_stack);
    println!("app_buffer  = {:#?}", ast_builder.app_buffer);
    println!("------------------------------");
    Ok(())
}
