use {
    olus::{
        Files,
        ir::Atom,
        parser::{ElementRef, Kind, Node, NodeExt, TokenExt, compile, parse},
    },
    std::path::PathBuf,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("./examples/test.olus");

    let mut files = Files::new();
    let file_id = files.insert(path)?;
    let source = files[file_id].contents();
    let root = parse(source);

    pretty_print(&root, 1);

    let program = compile(&root);

    for proc in program.procedures {
        for (i, arg) in proc.arguments.iter().enumerate() {
            eprint!("x{}", arg.id);
            if i != proc.arguments.len() - 1 {
                eprint!(" ");
            }
        }
        eprint!(":");
        for a in proc.body {
            eprint!(" ");
            match a {
                Atom::Number { value, .. } => eprint!("{value}"),
                Atom::String { value, .. } => eprint!("{value}"),
                Atom::Reference { id, .. } => eprint!("x{id}"),
            }
        }
        eprintln!();
    }

    Ok(())
}

fn pretty_print(node: &Node, indent_level: usize) {
    let indent = "  ".repeat(indent_level);
    eprint!(
        "{:>4}..{:<4}{indent}{:?}",
        usize::from(node.text_range().start()),
        usize::from(node.text_range().end()),
        node.kind()
    );
    if node.kind() == Kind::Proc {
        eprint!(" {:?}", node.call());
    }
    eprintln!();

    // Recursively print syntax child nodes
    for child in node.children_with_tokens() {
        if !child.kind().is_syntax() {
            continue;
        }
        match child {
            ElementRef::Node(node) => pretty_print(node, indent_level + 1),
            ElementRef::Token(token) => {
                eprint!(
                    "{:>4}..{:<4}{indent}  {:?} {:?}",
                    usize::from(token.text_range().start()),
                    usize::from(token.text_range().end()),
                    token.kind(),
                    token.text(),
                );
                if token.is_reference() {
                    eprint!(" {:?}", token.resolve());
                }
                if token.is_binder() {
                    eprint!(" BINDER");
                }
                eprintln!();
            }
        }
    }
}
