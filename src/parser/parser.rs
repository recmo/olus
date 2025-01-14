//! Ties Logos, Chomsky and CSTree together in a parser.
//! See <https://github.com/spreadsheet-lang/spreadsheet/blob/main/lang/src/parser.rs>
use {
    super::{Lexer, Node, Span},
    ariadne::{Color, Label, Report, ReportKind, Source},
    chumsky::{
        extension::v1::{Ext, ExtParser},
        input::{BoxedStream, Cursor, InputRef, MapExtra, MappedInput, Stream, ValueInput},
        inspector::Inspector,
        prelude::*,
    },
    core::marker::PhantomData,
    cstree::{
        build::{Checkpoint, GreenNodeBuilder},
        green::GreenNode,
    },
};

// Type and trait aliases for the Chumsky parser to make things more concrete.
type CstError<'s> = Rich<'s, Node>;
type CstExtra<'s, 'c> = extra::Full<CstError<'s>, CstState<'c>, ()>;
type CstInput<'s> = MappedInput<Node, Span, Stream<Lexer<'s>>, fn((Node, Span)) -> (Node, Span)>;
type CstMapExtra<'s, 'c, 'a> = MapExtra<'s, 'a, CstInput<'s>, CstExtra<'s, 'c>>;
type CstCursor<'s, 'a> = Cursor<'s, 'a, CstInput<'s>>;
type CstCheckpoint<'s, 'a> = chumsky::input::Checkpoint<'s, 'a, CstInput<'s>, Checkpoint>;

trait CstParser<'s, 'c: 's, Output = ()>: Parser<'s, CstInput<'s>, Output, CstExtra<'s, 'c>> {}
impl<'s, 'c: 's, O, P: Parser<'s, CstInput<'s>, O, CstExtra<'s, 'c>>> CstParser<'s, 'c, O> for P {}

/// Parser state containing CST builder.
struct CstState<'c> {
    builder: GreenNodeBuilder<'c, 'c, Node>,
}

#[derive(Clone, Copy)]
struct CstLeafExt<'s, 'c: 's, P: CstParser<'s, 'c, Node>> {
    parser:   P,
    _phantom: PhantomData<(&'s (), &'c ())>,
}

#[derive(Clone, Copy)]
struct CstNodeExt<'s, 'c: 's, P: CstParser<'s, 'c>> {
    node:     Node,
    parser:   P,
    _phantom: PhantomData<(&'s (), &'c ())>,
}

/// Inspector for the Chumsky parser to build the CST.
impl<'s, 'c: 's> Inspector<'s, CstInput<'s>> for CstState<'c> {
    type Checkpoint = Checkpoint;

    fn on_token(&mut self, token: &Node) {}

    fn on_save<'parse>(&self, cursor: &CstCursor<'s, 'parse>) -> Checkpoint {
        self.builder.checkpoint()
    }

    fn on_rewind<'parse>(&mut self, marker: &CstCheckpoint<'s, 'parse>) {
        self.builder.revert_to(*marker.inspector());
    }
}

impl<'s, 'c: 's, P: CstParser<'s, 'c, Node>> ExtParser<'s, CstInput<'s>, (), CstExtra<'s, 'c>>
    for CstLeafExt<'s, 'c, P>
{
    fn parse<'parse>(
        &self,
        inp: &mut InputRef<'s, 'parse, CstInput<'s>, CstExtra<'s, 'c>>,
    ) -> Result<(), CstError<'s>> {
        let node = inp.parse(&self.parser)?;
        eprintln!("token {:?}", node);
        inp.state().builder.token(node, "KK");
        Ok(())
    }
}

impl<'s, 'c: 's, P: CstParser<'s, 'c>> ExtParser<'s, CstInput<'s>, (), CstExtra<'s, 'c>>
    for CstNodeExt<'s, 'c, P>
{
    fn parse<'parse>(
        &self,
        inp: &mut InputRef<'s, 'parse, CstInput<'s>, CstExtra<'s, 'c>>,
    ) -> Result<(), CstError<'s>> {
        let checkpoint = inp.state().builder.checkpoint();
        eprintln!("node start {:?} {checkpoint:?}", self.node);
        inp.parse(&self.parser)?;
        let mut builder = &mut inp.state().builder;
        builder.start_node_at(checkpoint, self.node);
        builder.finish_node();
        eprintln!("node finish {:?} {checkpoint:?}", self.node);
        Ok(())
    }
}

trait ParserExt<'s, 'c: 's>: CstParser<'s, 'c> + Sized {
    fn node(self, node: Node) -> Ext<CstNodeExt<'s, 'c, Self>> {
        Ext(CstNodeExt {
            node,
            parser: self,
            _phantom: PhantomData,
        })
    }
}

impl<'s, 'c: 's, P: CstParser<'s, 'c>> ParserExt<'s, 'c> for P {}

fn leaf<'s, 'c: 's>(node: Node) -> impl CstParser<'s, 'c> + Clone {
    Ext(CstLeafExt {
        parser:   just(node),
        _phantom: PhantomData,
    })
}

fn leaf_to<'s, 'c: 's>(node: Node, to: Node) -> impl CstParser<'s, 'c> {
    Ext(CstLeafExt {
        parser:   just(node).to(to),
        _phantom: PhantomData,
    })
}

/// Chumsky grammar for the Olus language.
fn parser<'source, 'cache: 'source>() -> impl CstParser<'source, 'cache> {
    let whitespace = leaf(Node::Whitespace);
    let newline = leaf(Node::Newline);
    let identifier = leaf(Node::Identifier);

    let atom = choice((
        leaf(Node::Identifier),
        leaf(Node::Number),
        leaf(Node::String),
    ));

    let expression = recursive(|expression| {
        let call = expression
            .clone()
            .separated_by(leaf(Node::Whitespace))
            .delimited_by(leaf(Node::ParenOpen), leaf(Node::ParenClose))
            .node(Node::Call);

        let procedure = leaf(Node::Identifier)
            .separated_by(leaf(Node::Whitespace))
            .then_ignore(leaf(Node::Colon).padded_by(leaf(Node::Whitespace).or_not()))
            .then_ignore(expression.separated_by(leaf(Node::Whitespace)))
            .delimited_by(leaf(Node::ParenOpen), leaf(Node::ParenClose))
            .node(Node::Proc);

        choice((atom.clone(), call, procedure))
    });

    // let call = expression.separated_by(just(Node::Whitespace)).ignored();

    // let procedure = just(Node::Identifier)
    //     .separated_by(just(Node::Whitespace))
    //     .then(just(Node::Colon).padded_by(just(Node::Whitespace).or_not()))
    //     .then(call.clone().or_not())
    //     .ignored();

    // let statement = procedure.or(call).then(just(Node::Newline)).ignored();

    // let block = recursive(|block| {
    //     choice((
    //         statement,
    //         block.delimited_by(just(Node::Indent), just(Node::Dedent)),
    //     ))
    //     .repeated()
    //     .ignored()
    // });

    expression
        .separated_by(whitespace)
        .then_ignore(newline)
        .node(Node::Block)
}

pub fn parse(source: &str) -> GreenNode {
    // Construct a (token, span) stream from the lexer.
    let lexer = Lexer::new(source);
    let end_of_input = Span::splat(source.len());
    let token_stream: CstInput = Stream::from_iter(lexer).map(end_of_input, |(t, s)| (t, s));

    // Construct a builder to build the CST.
    let mut state = CstState {
        builder: GreenNodeBuilder::<Node>::new(),
    };
    state.builder.start_node(Node::Root);

    let result = parser()
        .parse_with_state(token_stream, &mut state)
        .into_result();
    match result {
        Ok(_) => {}
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

    state.builder.finish_node();
    let (root, node_cache) = state.builder.finish();
    root
}
