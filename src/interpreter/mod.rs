use {
    crate::ir::{Atom, Program},
    std::{collections::HashMap, iter::zip},
};

#[derive(Clone, Debug)]
pub enum Value<B> {
    Builtin(B),
    Number(u64),
    String(String),
    Closure(u32, Context<B>),
}

#[derive(Clone, Debug)]
pub struct Context<B>(HashMap<u32, Value<B>>);

pub fn evaluate<B: Clone, R, F: FnOnce(&Program<B>, &B, &[Value<B>]) -> R>(
    program: &Program<B>,
    builtin: F,
    call: &[Value<B>],
) -> R {
    match &call[0] {
        Value::Builtin(b) => builtin(program, b, &call[1..]),
        Value::Closure(id, context) => {
            // Find the associated proc in the program
            let proc = program
                .procedures
                .iter()
                .find(|proc| proc.arguments[0].id == *id)
                .unwrap();

            // Pair up arguments and add to context
            let mut context = context.clone();
            for (arg, val) in zip(proc.arguments.iter(), call) {
                context.0.insert(arg.id, val.clone());
            }

            // Evaluate the body with context
            let body = proc
                .body
                .iter()
                .map(|atom| match atom {
                    Atom::Builtin { builtin, .. } => Value::Builtin(builtin.clone()),
                    Atom::Number { value, .. } => Value::Number(*value),
                    Atom::String { value, .. } => Value::String(value.clone()),
                    Atom::Reference { id, .. } => context
                        .0
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| Value::Closure(*id, context.clone())),
                })
                .collect::<Vec<_>>();

            // Evaluate
            evaluate(program, builtin, &body)
        }
        _ => panic!("Can not evaluate non-closure."),
    }
}

impl<B> Context<B> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}
