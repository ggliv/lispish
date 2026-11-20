use crate::error::{LispishError, Result};
use crate::parser::Expr;
use crate::runtime::*;

fn sf_quote(args: &[Expr], heap: &mut Heap) -> Result<Value> {
    if args.len() != 1 {
        return Err(LispishError::ArityMismatch {
            name: "quote".into(),
            expected: 1,
            found: args.len(),
        });
    }
    match &args[0] {
        Expr::Symbol(s) => Ok(heap.alloc_val(HeapObj::Symbol(s.into()))),
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::String(s) => Ok(heap.alloc_val(HeapObj::String(s.into()))),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::List(exprs) => {
            let mut values = Vec::with_capacity(exprs.len());
            for expr in exprs {
                values.push(sf_quote(std::slice::from_ref(expr), heap)?);
            }
            Ok(heap.alloc_val(HeapObj::List(values)))
        }
    }
}

fn sf_if(args: &[Expr], env: ObjId, rt: &mut Runtime) -> Result<Value> {
    if !matches!(args.len(), 2 | 3) {
        return Err(LispishError::ArityMismatch {
            name: "if".into(),
            expected: 2,
            found: args.len(),
        });
    }

    let test = &args[0];
    let if_true = &args[1];
    let if_false = &args.get(2);

    let test_val = eval(test, env, rt)?;
    let is_truthy = match test_val {
        Value::Obj(id) => match rt.heap.get(id) {
            HeapObj::List(l) => !l.is_empty(),
            _ => true,
        },
        Value::Bool(false) => false,
        _ => true,
    };

    if is_truthy {
        eval(if_true, env, rt)
    } else if let Some(false_expr) = if_false {
        eval(false_expr, env, rt)
    } else {
        Ok(rt.heap.nil())
    }
}

fn sf_lambda(args: &[Expr], env: ObjId, heap: &mut Heap) -> Result<Value> {
    if args.len() < 2 {
        return Err(LispishError::ArityMismatch {
            name: "lambda".into(),
            expected: 2,
            found: args.len(),
        });
    }

    let Expr::List(params_exprs) = &args[0] else {
        return Err(LispishError::TypeMismatch);
    };

    let mut params = Vec::new();
    for param_expr in params_exprs {
        match param_expr {
            Expr::Symbol(s) => params.push(s.clone()),
            _ => return Err(LispishError::TypeMismatch),
        }
    }

    let body = Vec::from(&args[1..]);

    Ok(heap.alloc_val(HeapObj::Closure { params, body, env }))
}

fn sf_define(args: &[Expr], env: ObjId, rt: &mut Runtime) -> Result<Value> {
    if args.len() != 2 {
        return Err(LispishError::ArityMismatch {
            name: "define".into(),
            expected: 2,
            found: args.len(),
        });
    }

    let Expr::Symbol(name) = &args[0] else {
        return Err(LispishError::TypeMismatch);
    };

    let val = eval(&args[1], env, rt)?;

    let HeapObj::Env {
        bindings: env_bindings,
        ..
    } = rt.heap.get_mut(env)
    else {
        panic!("Expected env at heap ID {}", env.0);
    };

    env_bindings.insert(name.clone(), val);

    Ok(val)
}

fn apply(f: Value, args: &[Value], rt: &mut Runtime) -> Result<Value> {
    let heap = &mut rt.heap;

    match f {
        Value::Builtin(t) => (t.resolve())(args, rt),
        Value::Obj(id) => {
            let HeapObj::Closure { params, body, env } = heap.get(id) else {
                return Err(LispishError::TypeMismatch);
            };

            if params.len() != args.len() {
                return Err(LispishError::ArityMismatch {
                    name: "<lambda>".into(),
                    expected: params.len(),
                    found: args.len(),
                });
            }

            let body = body.clone();
            let new_env = heap.alloc(HeapObj::Env {
                bindings: params.iter().cloned().zip(args.iter().copied()).collect(),
                parent: Some(*env),
            });

            let mut result = heap.nil();
            for expr in body {
                result = eval(&expr, new_env, rt)?;
            }
            Ok(result)
        }
        _ => Err(LispishError::TypeMismatch),
    }
}

pub fn eval(expr: &Expr, env: ObjId, rt: &mut Runtime) -> Result<Value> {
    let heap = &mut rt.heap;

    match expr {
        Expr::Symbol(s) => heap.env_get(env, s),
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::String(s) => Ok(rt.heap.alloc_val(HeapObj::String(s.clone()))),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::List(exprs) => {
            let Some((head, tail)) = exprs.split_first() else {
                return Err(LispishError::EmptyApplication);
            };

            if let Expr::Symbol(sym) = head {
                match sym.as_str() {
                    "quote" => return sf_quote(tail, &mut rt.heap),
                    "if" => return sf_if(tail, env, rt),
                    "lambda" => return sf_lambda(tail, env, &mut rt.heap),
                    "define" => return sf_define(tail, env, rt),
                    _ => (),
                }
            }

            let f = eval(head, env, rt)?;
            let args = tail
                .iter()
                .map(|e| eval(e, env, rt))
                .collect::<Result<Vec<_>>>()?;
            apply(f, args.as_slice(), rt)
        }
    }
}
