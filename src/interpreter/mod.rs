use crate::ir::{Atom, Program};

#[derive(Clone, Debug)]
pub enum Value<B> {
    Builtin(B),
    Number(u64),
    String(String),
    Closure(u32, Vec<Value<B>>),
}

pub fn evaluate<B: Clone, R, F: FnOnce(&Program<B>, &B, &[Value<B>]) -> R>(
    program: &Program<B>,
    builtin: F,
    call: &[Value<B>],
) -> R {
    match &call[0] {
        Value::Builtin(b) => builtin(program, b, &call[1..]),
        Value::Closure(id, closure) => {
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
            let body = proc
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
                            let mut new_closure = vec![];
                            for id in &new_proc.closure {
                                if let Some(i) = proc.closure.iter().position(|&cid| cid == *id) {
                                    new_closure.push(closure[i].clone());
                                    continue;
                                }
                                if let Some(i) = proc.arguments.iter().position(|arg| arg.id == *id)
                                {
                                    new_closure.push(call[i].clone());
                                    continue;
                                }
                                panic!("Unresolved variable {id} in closure creation.")
                            }
                            return Value::Closure(*id, new_closure);
                        }

                        panic!("Unresolved variable.")
                    }
                })
                .collect::<Vec<_>>();

            // Evaluate
            evaluate(program, builtin, &body)
        }
        _ => panic!("Can not evaluate non-closure."),
    }
}
