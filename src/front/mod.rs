//! Parser for the Oluś language.
//! See [parser] for the grammar and [Node] for the lexer.

mod compiler;
mod cst_parser;
mod grammar;
mod indentation;
mod lexer;
mod syntax;

pub use {
    self::{
        compiler::compile,
        lexer::Kind,
        syntax::{NodeExt, TokenExt},
    },
    chumsky::span::SimpleSpan as Span,
};
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
        syntax::{ResolvedElement, ResolvedElementRef, ResolvedNode, ResolvedToken, SyntaxNode},
    },
    yansi::Color,
};

// Concrete syntax tree types.
pub type Node = ResolvedNode<Kind>;
pub type Token = ResolvedToken<Kind>;
pub type Element = ResolvedElement<Kind>;
pub type ElementRef<'a> = ResolvedElementRef<'a, Kind>;

/// Parse the given source code into a concrete syntax tree.
#[must_use]
pub fn parse(source: &str) -> Node {
    // Construct a (token, span) stream from the lexer.
    let lexer = Lexer::new(source);
    let end_of_input = Span::splat(source.len());
    let token_stream: CstInput = Stream::from_iter(lexer).map(end_of_input, |(t, s)| (t, s));

    // Construct a builder to build the CST.
    let builder = GreenNodeBuilder::<Kind>::new();
    let mut state = CstState { source, builder };
    state.builder.start_node(Kind::Block); // Root node is a block

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
