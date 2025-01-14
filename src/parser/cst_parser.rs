//! Ties [logos], [chumsky] and [cstree] together in a parser.
//! See <https://github.com/spreadsheet-lang/spreadsheet/blob/main/lang/src/parser.rs>
use {
    super::{Lexer, Node, Span},
    chumsky::{
        extension::v1::{Ext, ExtParser},
        input::{Cursor, InputRef, MappedInput, Stream},
        inspector::Inspector,
        prelude::*,
    },
    core::marker::PhantomData,
    cstree::build::{Checkpoint, GreenNodeBuilder},
};

// Type and trait aliases for the Chumsky parser to make things more concrete.
pub(super) type CstError<'s> = Rich<'s, Node>;
pub(super) type CstExtra<'s, 'c> = extra::Full<CstError<'s>, CstState<'s, 'c>, ()>;
pub(super) type CstInput<'s> =
    MappedInput<Node, Span, Stream<Lexer<'s>>, fn((Node, Span)) -> (Node, Span)>;
pub(super) type CstCursor<'s, 'a> = Cursor<'s, 'a, CstInput<'s>>;
pub(super) type CstCheckpoint<'s, 'a> =
    chumsky::input::Checkpoint<'s, 'a, CstInput<'s>, Checkpoint>;

pub(super) trait CstParser<'s, 'c: 's, Output = ()>:
    Parser<'s, CstInput<'s>, Output, CstExtra<'s, 'c>>
{
}
impl<'s, 'c: 's, O, P: Parser<'s, CstInput<'s>, O, CstExtra<'s, 'c>>> CstParser<'s, 'c, O> for P {}

/// Parser state containing CST builder.
pub(super) struct CstState<'s, 'c> {
    pub(super) source:  &'s str,
    pub(super) builder: GreenNodeBuilder<'c, 'c, Node>,
}

/// Generate a `GreenToken` from a parser that outputs the token kind.
#[derive(Clone, Copy)]
pub(super) struct CstLeafExt<'s, 'c: 's, P: CstParser<'s, 'c, Node>> {
    parser:   P,
    _phantom: PhantomData<(&'s (), &'c ())>,
}

/// Generate a `GreenNode` of the provided kind containing the parse.
#[derive(Clone, Copy)]
pub(super) struct CstNodeExt<'s, 'c: 's, P: CstParser<'s, 'c>> {
    node:     Node,
    parser:   P,
    _phantom: PhantomData<(&'s (), &'c ())>,
}

pub(super) trait ParserExt<'s, 'c: 's>: CstParser<'s, 'c> + Sized {
    fn node(self, node: Node) -> Ext<CstNodeExt<'s, 'c, Self>> {
        Ext(CstNodeExt {
            node,
            parser: self,
            _phantom: PhantomData,
        })
    }
}

impl<'s, 'c: 's, P: CstParser<'s, 'c>> ParserExt<'s, 'c> for P {}

pub(super) fn token<'s, 'c: 's>(node: Node) -> impl CstParser<'s, 'c> + Clone {
    Ext(CstLeafExt {
        parser:   just(node),
        _phantom: PhantomData,
    })
}

/// Inspector for the Chumsky parser to build the CST.
impl<'s, 'c: 's> Inspector<'s, CstInput<'s>> for CstState<'s, 'c> {
    type Checkpoint = Checkpoint;

    fn on_token(&mut self, _token: &Node) {}

    fn on_save<'parse>(&self, _cursor: &CstCursor<'s, 'parse>) -> Checkpoint {
        self.builder.checkpoint()
    }

    fn on_rewind<'parse>(&mut self, marker: &CstCheckpoint<'s, 'parse>) {
        self.builder.revert_to(*marker.inspector());
    }
}

/// Parser extension to create `GreenToken`s
impl<'s, 'c: 's, P: CstParser<'s, 'c, Node>> ExtParser<'s, CstInput<'s>, (), CstExtra<'s, 'c>>
    for CstLeafExt<'s, 'c, P>
{
    fn parse<'parse>(
        &self,
        inp: &mut InputRef<'s, 'parse, CstInput<'s>, CstExtra<'s, 'c>>,
    ) -> Result<(), CstError<'s>> {
        let before = inp.cursor();
        let node = inp.parse(&self.parser)?;
        let span = inp.span_since(&before);
        let text = &inp.state().source[span.start..span.end];
        inp.state().builder.token(node, text);
        Ok(())
    }
}

/// Parser extension to create `GreenNode`s
impl<'s, 'c: 's, P: CstParser<'s, 'c>> ExtParser<'s, CstInput<'s>, (), CstExtra<'s, 'c>>
    for CstNodeExt<'s, 'c, P>
{
    fn parse<'parse>(
        &self,
        inp: &mut InputRef<'s, 'parse, CstInput<'s>, CstExtra<'s, 'c>>,
    ) -> Result<(), CstError<'s>> {
        // eprintln!("> {:?}", self.node);
        let checkpoint = inp.state().builder.checkpoint();
        inp.parse(&self.parser)?;
        let builder = &mut inp.state().builder;
        builder.start_node_at(checkpoint, self.node);
        builder.finish_node();
        // eprintln!("< {:?}", self.node);
        Ok(())
    }
}
