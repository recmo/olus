use {
    cstree::{syntax::SyntaxNode, util::NodeOrToken},
    olus::{Files, parser::Node},
    std::path::PathBuf,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("./examples/test.olus");

    let mut files = Files::new();
    let file_id = files.insert(path)?;
    let source = files[file_id].contents();
    let root = olus::parser::parser::parse(source);

    let root = SyntaxNode::<Node>::new_root(root);

    pretty_print(&root, 0);

    Ok(())
}

fn pretty_print(node: &SyntaxNode<Node>, indent_level: usize) {
    let indent = "  ".repeat(indent_level);
    println!("{indent}{:?}", node.kind());

    // Recursively print all child nodes
    for child in node.children_with_tokens() {
        match child {
            NodeOrToken::Node(node) => pretty_print(node, indent_level + 1),
            NodeOrToken::Token(token) => {
                eprintln!("{indent}  Token {:?}", token.kind());
            }
        }
    }
}
