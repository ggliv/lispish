use std::{error::Error, io::Write};

use lispish::{
    error::LispishError,
    eval,
    lexer::{self, Token},
    parser,
    runtime::Runtime,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut rt = Runtime::with_print(Box::new(|s| print!("{}", s)));

    let mut line = String::new();
    let mut tokens: Vec<Token> = Vec::new();

    loop {
        if tokens.is_empty() && line.is_empty() {
            print!("lispish> ");
        } else if line.is_empty() {
            print!("  ... ");
        }
        std::io::stdout().flush()?;

        std::io::stdin().read_line(&mut line)?;

        match lexer::lex(&line) {
            Ok(new_tokens) => {
                tokens.extend(new_tokens);
                line.clear();
            }
            Err(LispishError::UnterminatedString(_)) => continue,
            Err(e) => {
                eprintln!("Lex error: {e}");
                line.clear();
                tokens.clear();
                continue;
            }
        }

        match parser::parse(&tokens) {
            Ok(ast) => {
                for expr in ast {
                    match eval::eval(&expr, rt.global_env, &mut rt) {
                        Ok(val) => println!("=> {}", val.fmt_with(&rt.heap)),
                        Err(e) => eprintln!("Eval error: {e}"),
                    }
                    rt.gc();
                }
                tokens.clear();
            }
            Err(LispishError::UnexpectedEof) => continue,
            Err(e) => {
                eprintln!("Parse error: {e}");
                tokens.clear();
            }
        }
    }
}
