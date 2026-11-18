mod builtins;
mod error;
mod eval;
mod lexer;
mod parser;

use std::{collections::HashMap, error::Error, io};

use crate::{
    builtins::BUILTINS,
    eval::{Heap, HeapObj, ObjId, Value},
};

fn init_global_env(heap: &mut Heap) -> ObjId {
    let builtin_bindings: HashMap<_, _> = BUILTINS
        .iter()
        .map(|b| (b.0.to_string(), Value::Builtin(b.2, b.1)))
        .collect();

    heap.alloc(HeapObj::Env {
        bindings: builtin_bindings,
        parent: None,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = io::read_to_string(io::stdin())?;
    let tokens = lexer::lex(&input)?;
    let ast = parser::parse(tokens)?;

    let mut heap = Heap::default();
    let global_env = init_global_env(&mut heap);

    for expr in ast {
        eval::eval(&expr, global_env, &mut heap)?;
        heap.gc([global_env]);
    }

    println!("Final heap state: {:#?}", heap);

    Ok(())
}
