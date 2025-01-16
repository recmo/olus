use {
    cstree::{syntax::ResolvedNode, util::NodeOrToken},
    olus::{
        Files,
        parser::{Node, ResolvedTokenExt, parse},
    },
    std::path::PathBuf,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("./examples/test.olus");

    let mut files = Files::new();
    let file_id = files.insert(path)?;
    let source = files[file_id].contents();
    let root = parse(source);

    eprintln!("{root:#?}");

    pretty_print(&root, 1);

    Ok(())
}

fn pretty_print(node: &ResolvedNode<Node>, indent_level: usize) {
    let indent = "  ".repeat(indent_level);
    println!(
        "{:>4}..{:<4}{indent}{:?}",
        usize::from(node.text_range().start()),
        usize::from(node.text_range().end()),
        node.kind()
    );

    // Recursively print syntax child nodes
    for child in node.children_with_tokens() {
        if !child.kind().is_syntax() {
            continue;
        }
        match child {
            NodeOrToken::Node(node) => pretty_print(node, indent_level + 1),
            NodeOrToken::Token(token) => {
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
