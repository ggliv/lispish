mod builtins;
mod error;
mod eval;
mod lexer;
mod parser;
mod prelude;
mod runtime;

use std::{error::Error, io::Write};

use crate::{lexer::Token, prelude::LISPISH_PRELUDE, runtime::Runtime};

fn eval(source: impl AsRef<str>, rt: &mut Runtime) -> crate::error::Result<()> {
    let source = source.as_ref();
    let tokens = lexer::lex(source)?;
    let ast = parser::parse(&tokens)?;

    for expr in ast {
        eval::eval(&expr, rt.global_env, rt)?;
        rt.heap.gc([rt.global_env]);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut rt = Runtime::with_print(Box::new(|s| print!("{}", s)));
    eval(LISPISH_PRELUDE, &mut rt)?;

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
            Err(error::LispishError::UnterminatedString(_)) => continue,
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
                    rt.heap.gc(std::iter::once(rt.global_env));
                }
                tokens.clear();
            }
            Err(error::LispishError::UnexpectedEof) => continue,
            Err(e) => {
                eprintln!("Parse error: {e}");
                tokens.clear();
            }
        }
    }
}
