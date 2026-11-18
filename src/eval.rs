use std::collections::HashMap;

use crate::builtins::BuiltinFn;
use crate::error::{LispishError, Result};
use crate::parser::Expr;

#[derive(Default, Debug)]
pub struct Heap {
    slots: Vec<Option<HeapSlot>>,
    free_slots: Vec<ObjId>,
}

impl Heap {
    pub(crate) fn alloc(&mut self, obj: HeapObj) -> ObjId {
        let new = Some(HeapSlot { marked: false, obj });
        if let Some(free_slot) = self.free_slots.pop() {
            self.slots[free_slot.0] = new;
            ObjId(free_slot.0)
        } else {
            self.slots.push(new);
            ObjId(self.slots.len() - 1)
        }
    }

    pub(crate) fn alloc_val(&mut self, obj: HeapObj) -> Value {
        Value::Obj(self.alloc(obj))
    }

    pub(crate) fn get(&self, id: ObjId) -> &HeapObj {
        &self
            .slots
            .get(id.0)
            .and_then(|o| o.as_ref())
            .unwrap_or_else(|| panic!("Dangling object ID {}", id.0))
            .obj
    }

    pub(crate) fn get_mut(&mut self, id: ObjId) -> &mut HeapObj {
        &mut self
            .slots
            .get_mut(id.0)
            .and_then(|o| o.as_mut())
            .unwrap_or_else(|| panic!("Dangling object ID {}", id.0))
            .obj
    }

    fn env_get(&self, env: ObjId, name: &str) -> Result<Value> {
        let mut env = &Some(env);

        while let Some(e) = env {
            let HeapObj::Env { bindings, parent } = self.get(*e) else {
                panic!("Expected env object at ID {}", e.0);
            };

            if let Some(v) = bindings.get(name) {
                return Ok(*v);
            }

            env = parent;
        }

        Err(LispishError::UnboundSymbol(name.into()))
    }

    fn mark(&mut self, root: ObjId) {
        let mut to_mark = vec![root];

        while let Some(id) = to_mark.pop() {
            let slot = self
                .slots
                .get_mut(id.0)
                .and_then(|o| o.as_mut())
                .unwrap_or_else(|| panic!("Dangling object ID {}", id.0));
            if slot.marked {
                // don't get stuck in loops
                continue;
            }
            slot.marked = true;

            match &slot.obj {
                HeapObj::List(l) => {
                    for child in l {
                        if let Value::Obj(child_id) = child {
                            to_mark.push(*child_id);
                        }
                    }
                }
                HeapObj::Closure { env, .. } => to_mark.push(*env),
                HeapObj::Env {
                    bindings, parent, ..
                } => {
                    if let Some(p) = parent {
                        to_mark.push(*p);
                    }
                    for child in bindings.values() {
                        if let Value::Obj(child_id) = child {
                            to_mark.push(*child_id);
                        }
                    }
                }
                _ => (),
            }
        }
    }

    pub fn gc(&mut self, roots: impl IntoIterator<Item = ObjId>) {
        // mark and sweep

        for slot in self.slots.iter_mut().flatten() {
            slot.marked = false;
        }

        for root in roots {
            self.mark(root);
        }

        for (i, slot) in self.slots.iter_mut().enumerate() {
            if let Some(s) = slot
                && !s.marked
            {
                self.free_slots.push(ObjId(i));
                *slot = None;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjId(usize);

#[derive(Debug, Clone)]
struct HeapSlot {
    marked: bool,
    obj: HeapObj,
}

#[derive(Debug, Clone)]
pub enum HeapObj {
    Symbol(String),
    String(String),
    List(Vec<Value>),
    Closure {
        params: Vec<String>,
        body: Vec<Expr>,
        env: ObjId,
    },
    Env {
        bindings: HashMap<String, Value>,
        parent: Option<ObjId>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Builtin(BuiltinFn, crate::builtins::BuiltinType),
    Obj(ObjId),
}

impl Value {
    pub fn to_string(&self, heap: &Heap) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::Bool(true) => "#t".to_string(),
            Value::Bool(false) => "#f".to_string(),
            Value::Builtin(_, builtin_type) => format!("<builtin:{}>", builtin_type),
            Value::Obj(obj_id) => match heap.get(*obj_id) {
                HeapObj::Symbol(s) => s.clone(),
                HeapObj::String(s) => s.clone(),
                HeapObj::List(values) => format!(
                    "({})",
                    values
                        .iter()
                        .map(|v| v.to_string(heap))
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                HeapObj::Closure { .. } => "#<closure>".to_string(),
                HeapObj::Env { .. } => "#<env>".to_string(),
            },
        }
    }
}

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

fn sf_if(args: &[Expr], env: ObjId, heap: &mut Heap) -> Result<Value> {
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

    let test_val = eval(test, env, heap)?;
    let is_truthy = match test_val {
        Value::Obj(id) => match heap.get(id) {
            HeapObj::List(l) => !l.is_empty(),
            _ => true,
        },
        Value::Bool(false) => false,
        _ => true,
    };

    if is_truthy {
        eval(if_true, env, heap)
    } else if let Some(false_expr) = if_false {
        eval(false_expr, env, heap)
    } else {
        Ok(heap.alloc_val(HeapObj::List(Vec::new())))
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

fn sf_define(args: &[Expr], env: ObjId, heap: &mut Heap) -> Result<Value> {
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

    let val = eval(&args[1], env, heap)?;

    let HeapObj::Env {
        bindings: env_bindings,
        ..
    } = heap.get_mut(env)
    else {
        panic!("Expected env at heap ID {}", env.0);
    };

    env_bindings.insert(name.clone(), val);

    Ok(val)
}

fn apply(f: Value, args: &[Value], heap: &mut Heap) -> Result<Value> {
    match f {
        Value::Builtin(b, _) => b(args, heap),
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

            let mut result = heap.alloc_val(HeapObj::List(Vec::new()));
            for expr in body {
                result = eval(&expr, new_env, heap)?;
            }
            Ok(result)
        }
        _ => Err(LispishError::TypeMismatch),
    }
}

pub fn eval(expr: &Expr, env: ObjId, heap: &mut Heap) -> Result<Value> {
    match expr {
        Expr::Symbol(s) => heap.env_get(env, s),
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::String(s) => Ok(heap.alloc_val(HeapObj::String(s.clone()))),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::List(exprs) => {
            let Some((head, tail)) = exprs.split_first() else {
                return Err(LispishError::EmptyApplication);
            };

            if let Expr::Symbol(sym) = head {
                match sym.as_str() {
                    "quote" => return sf_quote(tail, heap),
                    "if" => return sf_if(tail, env, heap),
                    "lambda" => return sf_lambda(tail, env, heap),
                    "define" => return sf_define(tail, env, heap),
                    _ => (),
                }
            }

            let f = eval(head, env, heap)?;
            let args = tail
                .iter()
                .map(|e| eval(e, env, heap))
                .collect::<Result<Vec<_>>>()?;
            apply(f, args.as_slice(), heap)
        }
    }
}
