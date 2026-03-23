mod lexer;
mod tokenizer;
mod translation_unit;

use lexer::lexer;
use tokenizer::Tokenizer;
use translation_unit::TU;

fn main() {
    let buf = include_str!("../test.ln");
    let tu = TU {
        filename: "<test.ln>".into(),
        contents: buf.into(),
    };
    let tn = Tokenizer::new(&tu);
    println!("-- File: ---------------------");
    print!("{:#}", buf);
    println!("------------------------------");
    let tokens = lexer().run(tn).unwrap();
    for token in tokens {
        let unprintable = format!("{:?}", token);
        println!(
            "{:^16} | {}",
            format!("{:?}", &tu.contents[token.span.0..token.span.1]),
            unprintable
        );
    }
}
