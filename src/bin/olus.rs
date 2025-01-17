use {
    olus::{
        Files,
        front::{compile, parse, pretty_print_cst},
        interpreter::{Context, Value, evaluate},
        ir::{Program, pretty_print_ir},
    },
    std::path::PathBuf,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("./examples/test.olus");

    let mut files = Files::new();
    let file_id = files.insert(path)?;
    let source = files[file_id].contents();
    let root = parse(source);

    pretty_print_cst(&root, 1);

    let program = compile(source.to_string(), &root, builtin_resolve);

    pretty_print_ir(&program);

    // Find a Prcocedure called main.
    let Some(main) = program.procedures.iter().find(|p| {
        p.arguments
            .first()
            .and_then(|arg| program.resolve_name(arg.id))
            == Some("main")
    }) else {
        panic!("No procedure `main` found.");
    };
    let [main, _exit] = main.arguments.as_slice() else {
        panic!("Procedure `main` should have one arguments.");
    };

    // Construct an initial call for the virtual machine.
    evaluate(&program, builtin_eval, &[
        Value::Closure(main.id, Context::new()),
        Value::Builtin("exit"),
    ]);

    Ok(())
}

fn builtin_resolve(name: &str) -> Option<&'static str> {
    const BUILTINS: &[&str] = &["exit", "print", "add", "if", "is_zero", "sub", "mul"];
    BUILTINS.iter().copied().find(|b| b == &name)
}

fn builtin_eval(program: &Program<&str>, builtin: &&str, call: &[Value<&str>]) {
    match *builtin {
        "print" => {
            println!("> {:?}", call[0]);
            evaluate(program, builtin_eval, &[call[1].clone()])
        }
        "exit" => {
            println!("> Exit");
            return;
        }
        "add" => {
            let Value::Number(a) = call[0] else { panic!() };
            let Value::Number(b) = call[1] else { panic!() };
            evaluate(program, builtin_eval, &[
                call[2].clone(),
                Value::Number(a + b),
            ])
        }
        "sub" => {
            let Value::Number(a) = call[0] else { panic!() };
            let Value::Number(b) = call[1] else { panic!() };
            evaluate(program, builtin_eval, &[
                call[2].clone(),
                Value::Number(a - b),
            ])
        }
        "mul" => {
            let Value::Number(a) = call[0] else { panic!() };
            let Value::Number(b) = call[1] else { panic!() };
            evaluate(program, builtin_eval, &[
                call[2].clone(),
                Value::Number(a * b),
            ])
        }
        "is_zero" => {
            let Value::Number(a) = call[0] else { panic!() };
            evaluate(program, builtin_eval, &[
                call[1].clone(),
                Value::Number(if a == 0 { 1 } else { 0 }),
            ])
        }
        "if" => {
            let Value::Number(a) = call[0] else { panic!() };
            evaluate(program, builtin_eval, &[(if a == 1 {
                &call[1]
            } else {
                &call[2]
            })
            .clone()])
        }
        _ => unimplemented!(),
    }
}
