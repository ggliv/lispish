use lispish::{
    error::LispishError,
    eval,
    lexer::{self, Token},
    parser,
    runtime::{Heap, Runtime},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = lispishPrint)]
    fn lispish_print(s: &str);
}

#[wasm_bindgen]
struct Lispish {
    rt: Runtime,
    tokens: Vec<Token>,
    line: String,
}

#[wasm_bindgen]
impl Lispish {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            rt: Runtime::with_print(Box::new(lispish_print)),
            tokens: Vec::new(),
            line: String::new(),
        }
    }

    pub fn try_from_json(json: Option<String>) -> Self {
        let Some(json) = json else {
            return Self::new();
        };

        let Ok(heap) = serde_json::from_str::<Heap>(&json) else {
            // TODO notify the outside world if we get here
            return Self::new();
        };

        let mut rt = Runtime::with_print(Box::new(lispish_print));

        rt.heap = heap;
        // TODO also need to reset nil value. should be fine for now though since nil is always the
        // first thing added to heap.

        Self {
            rt,
            tokens: Vec::new(),
            line: String::new(),
        }
    }

    pub fn to_json(&self) -> String {
        match serde_json::to_string(&self.rt.heap) {
            Ok(j) => j,
            Err(_) => String::new(),
        }
    }

    pub fn prompt(&self) -> String {
        if self.tokens.is_empty() && self.line.is_empty() {
            "lispish> ".into()
        } else {
            "  ... ".into()
        }
    }

    pub fn eval_line(&mut self, input: &str) -> String {
        self.line.push_str(input);
        self.line.push('\n');

        match lexer::lex(&self.line) {
            Ok(new_tokens) => {
                self.tokens.extend(new_tokens);
                self.line.clear();
            }
            Err(LispishError::UnterminatedString(_)) => return String::new(),
            Err(e) => {
                self.line.clear();
                self.tokens.clear();
                return format!("Lex error: {e}\n");
            }
        }

        if self.tokens.is_empty() {
            return String::new();
        }

        match parser::parse(&self.tokens) {
            Ok(ast) => {
                let mut out = String::new();
                for expr in ast {
                    match eval::eval(&expr, self.rt.global_env, &mut self.rt) {
                        Ok(val) => {
                            out.push_str(&format!("=> {}\n", val.fmt_with(&self.rt.heap)));
                        }
                        Err(e) => {
                            out.push_str(&format!("Eval error: {e}\n"));
                            break;
                        }
                    }
                    self.rt.gc();
                }
                self.tokens.clear();
                out
            }
            Err(LispishError::UnexpectedEof) => String::new(),
            Err(e) => {
                self.tokens.clear();
                format!("Parse error: {e}\n")
            }
        }
    }
}
