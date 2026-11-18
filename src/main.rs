mod builtins;
mod error;
mod eval;
mod lexer;
mod parser;
mod prelude;

use std::{
    error::Error,
    io::Write,
};

use crate::{
    builtins::init_global_env,
    eval::{Heap, ObjId},
    lexer::Token,
    prelude::LISPISH_PRELUDE,
};

fn eval(source: impl AsRef<str>, global_env: ObjId, heap: &mut Heap) -> crate::error::Result<()> {
    let source = source.as_ref();
    let tokens = lexer::lex(source)?;
    let ast = parser::parse(&tokens)?;

    for expr in ast {
        eval::eval(&expr, global_env, heap)?;
        heap.gc([global_env]);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut heap = Heap::default();
    let global_env = init_global_env(&mut heap);

    eval(LISPISH_PRELUDE, global_env, &mut heap)?;

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
                    match eval::eval(&expr, global_env, &mut heap) {
                        Ok(val) => println!("=> {}", val.fmt_with(&heap)),
                        Err(e) => eprintln!("Eval error: {e}"),
                    }
                    heap.gc(std::iter::once(global_env));
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
