use {
    olus::{
        Files,
        front::{compile, parse, pretty_print_cst},
        interpreter::{Value, evaluate},
        ir::{Atom, Program, pretty_print_ir},
    },
    std::{mem::swap, path::PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("./examples/test.olus");

    let mut files = Files::new();
    let file_id = files.insert(path)?;
    let source = files[file_id].contents();
    let root = parse(source);

    pretty_print_cst(&root, 1);

    let mut program = compile(source.to_string(), &root, builtin_resolve);

    // Find a Prcocedure called main.
    let Some(main) = program.procedure_by_name("main") else {
        panic!("No procedure `main` found.");
    };
    let main_id = main.id();

    program.tree_shake(main_id);
    pretty_print_ir(&program);
    program.closure_analysis();

    let main = program.procedure_by_id(main_id).unwrap();
    let [main, _exit] = main.arguments.as_slice() else {
        panic!("Procedure `main` should have one arguments.");
    };

    // Construct an initial call for the virtual machine.
    evaluate(&program, builtin_eval, &[
        Value::Closure(main.id, vec![]),
        Value::Builtin("exit"),
    ]);

    Ok(())
}

fn builtin_resolve(name: &str) -> Option<&'static str> {
    const BUILTINS: &[&str] = &["exit", "print", "add", "if", "is_zero", "sub", "mul"];
    BUILTINS.iter().copied().find(|b| b == &name)
}

fn builtin_eval(program: &Program<&str>, call: &mut Vec<Value<&str>>) -> Option<()> {
    let Some(Value::Builtin(builtin)) = call.first() else {
        panic!()
    };
    let mut body = match *builtin {
        "print" => {
            println!("> {:?}", call[1]);
            vec![call[2].clone()]
        }
        "exit" => {
            println!("> Exit");
            return Some(());
        }
        "add" => {
            let Value::Number(a) = call[1] else { panic!() };
            let Value::Number(b) = call[2] else { panic!() };
            vec![call[3].clone(), Value::Number(a + b)]
        }
        "sub" => {
            let Value::Number(a) = call[1] else { panic!() };
            let Value::Number(b) = call[2] else { panic!() };
            vec![call[3].clone(), Value::Number(a - b)]
        }
        "mul" => {
            let Value::Number(a) = call[1] else { panic!() };
            let Value::Number(b) = call[2] else { panic!() };
            vec![call[3].clone(), Value::Number(a * b)]
        }
        "is_zero" => {
            let Value::Number(a) = call[1] else { panic!() };
            vec![call[2].clone(), Value::Number((a == 0).into())]
        }
        "if" => {
            let Value::Number(a) = call[1] else { panic!() };
            vec![(if a == 1 { &call[2] } else { &call[3] }).clone()]
        }
        _ => unimplemented!(),
    };
    swap(call, &mut body);
    None
}
