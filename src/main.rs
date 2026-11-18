mod builtins;
mod error;
mod eval;
mod lexer;
mod parser;
mod prelude;

use std::{collections::HashMap, error::Error, io};

use crate::{
    builtins::{BUILTINS, init_global_env},
    eval::{Heap, HeapObj, ObjId, Value}, prelude::LISPISH_PRELUDE,
};

fn eval(source: impl AsRef<str>, global_env: ObjId, heap: &mut Heap) -> crate::error::Result<()> {
    let source = source.as_ref();
    let tokens = lexer::lex(source)?;
    let ast = parser::parse(tokens)?;

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

    let input = io::read_to_string(io::stdin())?;

    eval(input, global_env, &mut heap)?;

    Ok(())
}
