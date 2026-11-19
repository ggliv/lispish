use std::collections::HashMap;

use crate::{
    builtins::{BUILTINS, BuiltinFn},
    error::{LispishError, Result},
    parser::Expr,
};

pub struct Runtime {
    pub heap: Heap,
    pub global_env: ObjId,
    pub on_print: Box<dyn FnMut(&str)>,
}

impl Runtime {
    /// Create a new runtime with builtins and prelude loaded
    pub fn with_print(on_print: Box<dyn FnMut(&str)>) -> Self {
        let mut heap = Heap::new();

        let builtin_bindings: HashMap<_, _> = BUILTINS
            .iter()
            .map(|b| (b.0.to_string(), Value::Builtin(b.2, b.1)))
            .collect();

        let global_env = heap.alloc(HeapObj::Env {
            bindings: builtin_bindings,
            parent: None,
        });

        Self {
            heap,
            global_env,
            on_print,
        }
    }
}

#[derive(Debug)]
pub struct Heap {
    slots: Vec<Option<HeapSlot>>,
    free_slots: Vec<ObjId>,
    nil_id: ObjId,
}

impl Heap {
    pub fn new() -> Self {
        let mut new_heap = Self {
            slots: Vec::default(),
            free_slots: Vec::default(),
            nil_id: ObjId(0),
        };

        new_heap.nil_id = new_heap.alloc(HeapObj::List(Vec::new()));

        new_heap
    }

    pub fn nil(&self) -> Value {
        Value::Obj(self.nil_id)
    }

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

    pub fn env_get(&self, env: ObjId, name: &str) -> Result<Value> {
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

        self.slots[self.nil_id.0]
            .as_mut()
            .expect("nil_id must point to an object")
            .marked = true;

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
pub struct ObjId(pub usize);

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
    pub fn fmt_with(&self, heap: &Heap) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::Bool(true) => "#t".to_string(),
            Value::Bool(false) => "#f".to_string(),
            Value::Builtin(_, builtin_type) => format!("<builtin:{}>", builtin_type),
            Value::Obj(obj_id) => match heap.get(*obj_id) {
                HeapObj::Symbol(s) => s.clone(),
                HeapObj::String(s) => format!("{s:?}"),
                HeapObj::List(values) => format!(
                    "({})",
                    values
                        .iter()
                        .map(|v| v.fmt_with(heap))
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                HeapObj::Closure { .. } => "#<closure>".to_string(),
                HeapObj::Env { .. } => "#<env>".to_string(),
            },
        }
    }
}
