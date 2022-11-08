// https://github.com/rust-analyzer/rowan/blob/master/examples/s_expressions.rs

mod name_resolution;
mod parser;
mod syntax;
mod syntax_kind;
mod token;

use self::{
    parser::Parser,
    syntax_kind::{Language, SyntaxKind},
    token::Token,
};
use rowan::{ast::AstNode, GreenNode};

pub use syntax::{Root, SyntaxNode};

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
