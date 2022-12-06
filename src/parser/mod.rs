// https://github.com/rust-analyzer/rowan/blob/master/examples/s_expressions.rs

mod parser;
pub mod syntax;
mod syntax_kind;
mod token;

use self::{
    parser::Parser,
    syntax::{Root, SyntaxNode},
    syntax_kind::{Language, SyntaxKind},
    token::Token,
};
use crate::{Diagnostic, FileId, Span};
use ariadne::{Color, Report, ReportKind};
use rowan::{ast::AstNode, GreenNode};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parse {
    pub green_node: GreenNode,
    pub errors:     Vec<Diagnostic>,
}

impl Parse {
    pub fn root(&self) -> syntax::Root {
        Root::cast(SyntaxNode::new_root(self.green_node.clone())).unwrap()
    }
}

pub fn parse(file_id: FileId, text: &str) -> Parse {
    let mut parser = Parser::new(file_id, text);
    parser.parse_root();
    parser.finish()
}

pub fn print(root: Root) {
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
    print(0, root.syntax());
}
