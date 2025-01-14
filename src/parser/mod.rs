// https://github.com/rust-analyzer/rowan/blob/master/examples/s_expressions.rs

mod lexer;
pub mod parser;
// pub mod syntax;

use {
    crate::{Diagnostic, FileId},
    ariadne::{Color, Report, ReportKind},
    std::ops::Range,
};
pub use {
    chumsky::span::SimpleSpan as Span,
    lexer::{Lexer, Node},
};

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub struct Parse {
//     pub green_node: GreenNode,
//     pub errors:     Vec<Diagnostic>,
// }

// impl Parse {
//     #[must_use]
//     pub fn root(&self) -> syntax::Root {
//         Root::cast(SyntaxNode::new_root(self.green_node.clone())).unwrap()
//     }
// }

// #[must_use]
// pub fn parse(file_id: FileId, text: &str) -> Parse {
//     let mut parser = Parser::new(file_id, text);
//     parser.parse_root();
//     parser.finish()
// }

// pub fn print(root: &Root) {
//     fn print(depth: usize, node: &SyntaxNode) {
//         println!("{:depth$}{:?}@{:?}", "", node.kind(), node.text_range());
//         let depth = depth + 4;
//         for child in node.children_with_tokens() {
//             match child {
//                 rowan::NodeOrToken::Node(node) => print(depth, &node),
//                 rowan::NodeOrToken::Token(token) => {
//                     println!(
//                         "{:depth$}{:?}@{:?}: {:?}",
//                         "",
//                         token.kind(),
//                         token.text_range(),
//                         token.text()
//                     );
//                 }
//             }
//         }
//     }
//     print(0, root.syntax());
// }
