mod lexer;
mod tokenizer;
mod translation_unit;

use lexer::lexer;
use tokenizer::Tokenizer;
use translation_unit::TU;

fn main() {
    let expr = "\\f x y => (f y) x";
    let tu = TU {
        contents: expr.into(),
        filename: "<stdin>".into(),
    };
    let tn = Tokenizer::new(&tu);
    let tokens = lexer().run(tn).unwrap();
    println!(" Expr := {}", expr);
    for token in tokens {
        let unprintable = format!("{:?}", token);
        println!(
            "{:^6} | {}",
            &tu.contents[token.span.0..token.span.1],
            unprintable
        );
    }
}
