use std::error::Error;
use std::path::Path;

use lispish::{
    error::LispishError,
    eval,
    lexer::{self, Token},
    parser,
    runtime::Runtime,
};
use rustyline::Helper;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};

struct ValidatorHelper;

impl Validator for ValidatorHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();

        let tokens = match lexer::lex(input) {
            Ok(t) => t,
            Err(LispishError::UnterminatedString(_)) => return Ok(ValidationResult::Incomplete),
            Err(_) => return Ok(ValidationResult::Valid(None)),
        };

        match parser::parse(&tokens) {
            Err(LispishError::UnexpectedEof) => Ok(ValidationResult::Incomplete),
            _ => Ok(ValidationResult::Valid(None)),
        }
    }
}
impl Highlighter for ValidatorHelper {}
impl Hinter for ValidatorHelper {
    type Hint = String;
}
impl Completer for ValidatorHelper {
    type Candidate = String;
}
impl Helper for ValidatorHelper {}

fn detect_editor() -> rustyline::EditMode {
    let Ok(editor) = std::env::var("VISUAL").or_else(|_| std::env::var("EDITOR")) else {
        return rustyline::EditMode::Emacs;
    };

    match Path::new(editor.trim())
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
    {
        "vi" | "vim" | "nvim" | "hx" | "kak" => rustyline::EditMode::Vi,
        _ => rustyline::EditMode::Emacs,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let editor = detect_editor();
    let mut rl =
        rustyline::Editor::with_config(rustyline::Config::builder().edit_mode(editor).build())?;
    rl.set_helper(Some(ValidatorHelper));

    let mut rt = Runtime::with_print(Box::new(|s| print!("{}", s)));

    let mut line = String::new();
    let mut tokens: Vec<Token> = Vec::new();

    loop {
        let input = rl.readline("lispish> ")?;

        match lexer::lex(&input) {
            Ok(new_tokens) => {
                tokens.extend(new_tokens);
                line.clear();
            }
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
            Err(e) => {
                eprintln!("Parse error: {e}");
                tokens.clear();
            }
        }
    }
}
