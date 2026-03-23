mod lexer;
mod tokenizer;
mod translation_unit;

use std::io::Read;

use lexer::lexer;
use tokenizer::Tokenizer;
use translation_unit::TU;

fn main() {
    let mut buf = r#"
add u v = plus v u

main = \f x y => do
  ux <- x
  ret (f y) ux
  where
    f y = add y
"#;
    // std::io::stdin().read_to_string(&mut buf).unwrap();
    let tu = TU {
        filename: "<stdin>".into(),
        contents: buf.into(),
    };
    let tn = Tokenizer::new(&tu);
    let tokens = lexer().run(tn).unwrap();
    println!(" Expr := {}", tu.contents);
    for token in tokens {
        let unprintable = format!("{:?}", token);
        println!(
            "{:^16} | {}",
            format!("{:?}", &tu.contents[token.span.0..token.span.1]),
            unprintable
        );
    }
}
