use std::fmt::Display;

use crate::error::{LispishError, Result};
use crate::runtime::{HeapObj, Runtime, Value};

pub type BuiltinFn = fn(&[Value], &mut Runtime) -> Result<Value>;

pub const BUILTINS: &[(&str, BuiltinType, BuiltinFn)] = &[
    ("cons", BuiltinType::Cons, cons),
    ("car", BuiltinType::Car, car),
    ("cdr", BuiltinType::Cdr, cdr),
    ("eq?", BuiltinType::Eq, eq),
    ("atom?", BuiltinType::Atom, atom),
    ("+", BuiltinType::Add, add),
    ("-", BuiltinType::Sub, sub),
    ("*", BuiltinType::Mul, mul),
    ("/", BuiltinType::Div, div),
    ("<", BuiltinType::Lt, lt),
    ("print", BuiltinType::Print, print),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinType {
    Cons,
    Car,
    Cdr,
    Eq,
    Atom,
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Print,
}

impl Display for BuiltinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BuiltinType::Cons => "cons",
                BuiltinType::Car => "car",
                BuiltinType::Cdr => "cdr",
                BuiltinType::Eq => "eq?",
                BuiltinType::Atom => "atom?",
                BuiltinType::Add => "+",
                BuiltinType::Sub => "-",
                BuiltinType::Mul => "*",
                BuiltinType::Div => "/",
                BuiltinType::Lt => "<",
                BuiltinType::Print => "print",
            }
        )
    }
}

fn all_nums<'a>(args: impl IntoIterator<Item = &'a Value>) -> Option<Vec<f64>> {
    args.into_iter()
        .map(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        })
        .collect::<Option<_>>()
}

pub fn cons(args: &[Value], rt: &mut Runtime) -> Result<Value> {
    let [head, tail] = args else {
        return Err(LispishError::ArityMismatch {
            name: "cons".into(),
            expected: 2,
            found: args.len(),
        });
    };

    let Value::Obj(tail_id) = tail else {
        return Err(LispishError::TypeMismatch);
    };

    let HeapObj::List(tail_list) = rt.heap.get(*tail_id) else {
        return Err(LispishError::TypeMismatch);
    };

    let mut new_vec = Vec::with_capacity(tail_list.len() + 1);
    new_vec.push(*head);
    new_vec.extend_from_slice(tail_list);

    Ok(rt.heap.alloc_val(HeapObj::List(new_vec)))
}

pub fn car(args: &[Value], rt: &mut Runtime) -> Result<Value> {
    if args.len() != 1 {
        return Err(LispishError::ArityMismatch {
            name: "car".into(),
            expected: 1,
            found: args.len(),
        });
    }
    let first = args.first().unwrap();
    let Value::Obj(obj) = first else {
        return Err(LispishError::TypeMismatch);
    };
    let HeapObj::List(list) = rt.heap.get(*obj) else {
        return Err(LispishError::TypeMismatch);
    };

    list.first().copied().ok_or(LispishError::EmptyList)
}

pub fn cdr(args: &[Value], rt: &mut Runtime) -> Result<Value> {
    if args.len() != 1 {
        return Err(LispishError::ArityMismatch {
            name: "cdr".into(),
            expected: 1,
            found: args.len(),
        });
    }
    let first = args.first().unwrap();
    let Value::Obj(obj) = first else {
        return Err(LispishError::TypeMismatch);
    };
    let HeapObj::List(list) = rt.heap.get(*obj) else {
        return Err(LispishError::TypeMismatch);
    };
    if list.is_empty() {
        Err(LispishError::EmptyList)
    } else {
        Ok(rt.heap.alloc_val(HeapObj::List(list[1..].to_vec())))
    }
}

pub fn eq(args: &[Value], rt: &mut Runtime) -> Result<Value> {
    fn eq_pair(a: &Value, b: &Value, rt: &Runtime) -> bool {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Builtin(_, a), Value::Builtin(_, b)) => a == b,
            (Value::Obj(a), Value::Obj(b)) => match (rt.heap.get(*a), rt.heap.get(*b)) {
                (HeapObj::Symbol(a), HeapObj::Symbol(b)) => a == b,
                (HeapObj::String(a), HeapObj::String(b)) => a == b,
                (HeapObj::List(a), HeapObj::List(b)) => {
                    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| eq_pair(a, b, rt))
                }
                (_, _) => a == b,
            },
            (_, _) => false,
        }
    }

    Ok(Value::Bool(
        args.windows(2)
            .into_iter()
            .all(|w| eq_pair(&w[0], &w[1], rt)),
    ))
}

pub fn atom(args: &[Value], rt: &mut Runtime) -> Result<Value> {
    if args.is_empty() {
        return Err(LispishError::ArityMismatch {
            name: "atom".into(),
            expected: 1,
            found: args.len(),
        });
    }

    Ok(Value::Bool(args.iter().all(|a| match a {
        Value::Obj(id) => match rt.heap.get(*id) {
            HeapObj::List(l) => l.is_empty(),
            _ => true,
        },
        _ => true,
    })))
}

pub fn add(args: &[Value], _rt: &mut Runtime) -> Result<Value> {
    if args.len() < 2 {
        return Err(LispishError::ArityMismatch {
            name: "add".into(),
            expected: 2,
            found: args.len(),
        });
    };
    let nums: Vec<f64> = all_nums(args).ok_or(LispishError::TypeMismatch)?;

    Ok(Value::Number(nums.iter().fold(0., |acc, n| n + acc)))
}

pub fn sub(args: &[Value], _rt: &mut Runtime) -> Result<Value> {
    if args.len() < 2 {
        return Err(LispishError::ArityMismatch {
            name: "sub".into(),
            expected: 2,
            found: args.len(),
        });
    };
    let nums: Vec<f64> = all_nums(args).ok_or(LispishError::TypeMismatch)?;

    Ok(Value::Number(
        nums.iter().skip(1).fold(nums[0], |acc, n| acc - n),
    ))
}

pub fn mul(args: &[Value], _rt: &mut Runtime) -> Result<Value> {
    if args.len() < 2 {
        return Err(LispishError::ArityMismatch {
            name: "mul".into(),
            expected: 2,
            found: args.len(),
        });
    };
    let nums: Vec<f64> = all_nums(args).ok_or(LispishError::TypeMismatch)?;

    Ok(Value::Number(nums.iter().fold(1., |acc, n| n * acc)))
}

pub fn div(args: &[Value], _rt: &mut Runtime) -> Result<Value> {
    if args.len() < 2 {
        return Err(LispishError::ArityMismatch {
            name: "div".into(),
            expected: 2,
            found: args.len(),
        });
    };
    let nums: Vec<f64> = all_nums(args).ok_or(LispishError::TypeMismatch)?;

    Ok(Value::Number(
        nums.iter().skip(1).fold(nums[0], |acc, n| acc / n),
    ))
}

pub fn lt(args: &[Value], _rt: &mut Runtime) -> Result<Value> {
    if args.len() < 2 {
        return Err(LispishError::ArityMismatch {
            name: "lt".into(),
            expected: 2,
            found: args.len(),
        });
    };

    let nums: Vec<f64> = all_nums(args).ok_or(LispishError::TypeMismatch)?;

    Ok(Value::Bool(nums.windows(2).all(|w| w[0] < w[1])))
}

pub fn print(args: &[Value], rt: &mut Runtime) -> Result<Value> {
    for arg in args {
        match arg {
            Value::Obj(obj_id) => match rt.heap.get(*obj_id) {
                HeapObj::String(s) => (rt.on_print)(s),
                _ => (rt.on_print)(&arg.fmt_with(&rt.heap)),
            },
            _ => (rt.on_print)(&arg.fmt_with(&rt.heap)),
        }
    }
    Ok(args.last().copied().unwrap_or(rt.heap.nil()))
}
