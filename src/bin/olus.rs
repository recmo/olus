use ariadne::{Label, Report, ReportKind, Source};
use logos::Lexer;
use olus::{lexer::Token, parser::SyntaxNode};
use std::io;

fn main() {
    let source = std::fs::read_to_string("./examples/test.olus").unwrap();

    let mut lexer = Lexer::<Token>::new(&source);

    for (token, span) in lexer.spanned() {
        println!("{:?}: {:?}", token, &source[span]);
    }
    println!();

    let node = olus::parser::parse(&source).syntax();

    fn print(depth: usize, node: SyntaxNode, source: &str) {
        println!("{:depth$}{:?}@{:?}", "", node.kind(), node.text_range());
        let depth = depth + 1;
        for child in node.children_with_tokens() {
            match child {
                rowan::NodeOrToken::Node(node) => print(depth, node, source),
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
    print(0, node, &source);
}
