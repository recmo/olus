// https://github.com/rust-analyzer/rowan/blob/master/examples/s_expressions.rs

mod name_resolution;
mod parser;
pub mod syntax;
mod syntax_kind;
mod token;
mod unparser;

use self::{
    parser::Parser,
    syntax_kind::{Language, SyntaxKind},
    token::Token,
};
use rowan::{ast::AstNode, GreenNode};

pub use self::{
    syntax::{Root, SyntaxNode},
    unparser::unparse,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parse {
    pub green_node: GreenNode,
    pub errors:     Vec<String>,
}

impl Parse {
    pub fn root(&self) -> Root {
        Root::cast(SyntaxNode::new_root(self.green_node.clone())).unwrap()
    }
}

pub fn parse(text: &str) -> Parse {
    let mut parser = Parser::new(text);
    parser.parse_root();
    parser.finish()
}

pub fn print(node: &SyntaxNode) {
    fn print(depth: usize, node: &SyntaxNode) {
        println!("{:depth$}{:?}@{:?}", "", node.kind(), node.text_range());
        let depth = depth + 4;
        for child in node.children_with_tokens() {
            match child {
                rowan::NodeOrToken::Node(node) => print(depth, &node),
                rowan::NodeOrToken::Token(token) => {
                    println!(
                        "{:depth$}{:?}@{:?}: {:?}",
                        "",
                        token.kind(),
                        token.text_range(),
                        token.text()
                    )
                }
            }
        }
    }
    print(0, node);
}
