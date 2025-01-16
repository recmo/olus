//! Parser for the Oluś language.
//! See [parser] for the grammar and [Node] for the lexer.

pub mod cst_parser;
mod grammar;
mod indentation;
mod lexer;
mod syntax;

pub use {self::lexer::Node, chumsky::span::SimpleSpan as Span, syntax::ResolvedTokenExt};
use {
    self::{
        cst_parser::{CstInput, CstState},
        grammar::parser,
        indentation::Lexer,
    },
    ariadne::{Label, Report, ReportKind, Source},
    chumsky::{
        Parser,
        input::{Input, Stream},
    },
    cstree::{
        build::GreenNodeBuilder,
        syntax::{ResolvedNode, SyntaxNode},
    },
    yansi::Color,
};

#[must_use]
pub fn parse(source: &str) -> ResolvedNode<Node> {
    // Construct a (token, span) stream from the lexer.
    let lexer = Lexer::new(source);
    let end_of_input = Span::splat(source.len());
    let token_stream: CstInput = Stream::from_iter(lexer).map(end_of_input, |(t, s)| (t, s));

    // Construct a builder to build the CST.
    let builder = GreenNodeBuilder::<Node>::new();
    let mut state = CstState { source, builder };
    state.builder.start_node(Node::Root);

    // Parse the source and print errors.
    let result = parser()
        .parse_with_state(token_stream, &mut state)
        .into_result();
    match result {
        Ok(()) => {}
        Err(errs) => {
            for err in errs {
                Report::build(ReportKind::Error, err.span().into_range())
                    .with_code(3)
                    .with_message(err.to_string())
                    .with_label(
                        Label::new(err.span().into_range())
                            .with_message(err.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint(Source::from(source))
                    .unwrap();
            }
        }
    }

    // Complete and retrieve the root node.
    state.builder.finish_node();
    let (root, node_cache) = state.builder.finish();

    // GreenNodeBuilder::new() constructs a node cache and interner.
    let interner = node_cache.unwrap().into_interner().unwrap();
    SyntaxNode::new_root_with_resolver(root, interner)
}
