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
        Value::Builtin(100),
    ]);

    Ok(())
}

fn builtin_resolve(name: &str) -> Option<u8> {
    match name {
        "print" => 0,
        "if" => 1,
        "is_zero" => 2,
        "add" => 3,
        "sub" => 4,
        "mul" => 5,
        "div" => 6,
        "exit" => 100,
        _ => return None,
    }
    .into()
}

fn builtin_eval(program: &Program<u8>, builtin: &u8, call: &[Value<u8>]) {
    match *builtin {
        0 => {
            println!("> {:?}", call[0]);
        }
        100 => {
            println!("> Exit");
            return;
        }
        _ => unimplemented!(),
    }
    evaluate(program, builtin_eval, &[call[1].clone()])
}
