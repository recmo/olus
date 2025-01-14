use {
    cstree::{syntax::ResolvedNode, util::NodeOrToken},
    olus::{
        Files,
        parser::{Node, parse},
    },
    std::path::PathBuf,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("./examples/test.olus");

    let mut files = Files::new();
    let file_id = files.insert(path)?;
    let source = files[file_id].contents();
    let root = parse(source);

    pretty_print(&root, 0);

    Ok(())
}

fn pretty_print(node: &ResolvedNode<Node>, indent_level: usize) {
    let indent = "  ".repeat(indent_level);
    println!("{:?}\t{indent}{:?}", node.text_range(), node.kind());

    // Recursively print non-trivia child nodes
    for child in node.children_with_tokens() {
        if child.kind().is_trivia() {
            continue;
        }
        match child {
            NodeOrToken::Node(node) => pretty_print(node, indent_level + 1),
            NodeOrToken::Token(token) => {
                eprintln!(
                    "{:?}\t{indent}  Token {:?}\t\t{:?}",
                    token.text_range(),
                    token.kind(),
                    token.text()
                );
            }
        }
    }
}
