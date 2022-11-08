use ariadne::{Label, Report, ReportKind, Source};
use olus::parser::{parse, SyntaxNode};
use rowan::ast::AstNode;
use std::fs::read_to_string;

fn main() {
    let source = read_to_string("./examples/test.olus").unwrap();

    let root = parse(&source).root();

    fn print(depth: usize, node: &SyntaxNode, source: &str) {
        println!("{:depth$}{:?}@{:?}", "", node.kind(), node.text_range());
        let depth = depth + 4;
        for child in node.children_with_tokens() {
            match child {
                rowan::NodeOrToken::Node(node) => print(depth, &node, source),
                rowan::NodeOrToken::Token(token) => {
                    let start = usize::from(token.text_range().start());
                    let end = usize::from(token.text_range().end());
                    let range = start..end;
                    let text = &source[range];
                    println!(
                        "{:depth$}{:?}@{:?}: {:?}",
                        "",
                        token.kind(),
                        token.text_range(),
                        text
                    )
                }
            }
        }
    }
    print(0, root.syntax(), &source);
}
