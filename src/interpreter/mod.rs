use {
    crate::ir::{Atom, Program},
    std::{fmt::Debug, mem::swap},
};

#[derive(Clone, Debug)]
pub enum Value<B> {
    Builtin(B),
    Number(u64),
    String(String),
    Closure(u32, Vec<Value<B>>),
}

pub fn evaluate<B: Clone + Debug, R, F: FnMut(&Program<B>, &mut Vec<Value<B>>) -> Option<R>>(
    program: &Program<B>,
    mut builtin: F,
    call: &[Value<B>],
) -> R {
    let mut call = call.to_vec();
    loop {
        // eprint!("→ ");
        // for value in &call {
        //     eprint!("{:?} ", value);
        // }
        // eprintln!();
        if let Some(result) = iterate(program, &mut builtin, &mut call) {
            return result;
        }
    }
}

pub fn iterate<B: Clone, R, F: FnOnce(&Program<B>, &mut Vec<Value<B>>) -> Option<R>>(
    program: &Program<B>,
    builtin: F,
    call: &mut Vec<Value<B>>,
) -> Option<R> {
    assert!(!call.is_empty());

    // Builtins
    if let Value::Builtin(_) = &call[0] {
        return builtin(program, call);
    }

    // Closures
    let Value::Closure(id, closure) = &call[0] else {
        panic!("Can not evaluate non-closure.");
    };

    // Find the associated proc in the program
    let Some(proc) = program
        .procedures
        .iter()
        .find(|proc| proc.arguments[0].id == *id)
    else {
        panic!("Runtime error: Invalid closure.");
    };
    assert_eq!(proc.closure.len(), closure.len());

    // Evaluate the body with context
    let mut body = proc
        .body
        .iter()
        .map(|atom| match atom {
            Atom::Builtin { builtin, .. } => Value::Builtin(builtin.clone()),
            Atom::Number { value, .. } => Value::Number(*value),
            Atom::String { value, .. } => Value::String(value.clone()),
            Atom::Reference { id, .. } => {
                // Lookup in closure, then arguments
                if let Some(i) = proc.closure.iter().position(|&cid| cid == *id) {
                    return closure[i].clone();
                }

                // Lookup in arguments
                if let Some(i) = proc.arguments.iter().position(|arg| arg.id == *id) {
                    return call[i].clone();
                }

                // Check if proper name
                if let Some(new_proc) = program.procedure_by_id(*id) {
                    // Construct closure
                    let new_closure = new_proc
                        .closure
                        .iter()
                        .map(|id| {
                            if let Some(i) = proc.closure.iter().position(|&cid| cid == *id) {
                                return closure[i].clone();
                            }
                            if let Some(i) = proc.arguments.iter().position(|arg| arg.id == *id) {
                                return call[i].clone();
                            }
                            panic!("Unresolved variable {id} in closure creation.")
                        })
                        .collect();
                    return Value::Closure(*id, new_closure);
                }

                panic!("Unresolved variable.")
            }
        })
        .collect::<Vec<_>>();

    swap(call, &mut body);
    None
}
