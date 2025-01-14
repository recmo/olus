use {
    super::{
        Node,
        cst_parser::{CstParser, ParserExt, token},
    },
    chumsky::prelude::*,
};

/// Chumsky grammar for the Olus language.
#[must_use]
pub(super) fn parser<'source, 'cache: 'source>() -> impl CstParser<'source, 'cache> {
    #[allow(clippy::enum_glob_use)]
    use Node::*;

    let expression = recursive(|expression| {
        let atom = choice((token(Identifier), token(Number), token(String)));

        let call = expression
            .clone()
            .then_ignore(token(Whitespace).or_not())
            .repeated()
            .padded_by(token(Whitespace).or_not())
            .delimited_by(token(ParenOpen), token(ParenClose))
            .node(Call);

        let procedure = token(Identifier)
            .separated_by(token(Whitespace))
            .then_ignore(token(Colon).padded_by(token(Whitespace).or_not()))
            .then_ignore(expression.separated_by(token(Whitespace)))
            .padded_by(token(Whitespace).or_not())
            .delimited_by(token(ParenOpen), token(ParenClose))
            .node(Proc);

        choice((atom, call, procedure))
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
        .then_ignore(token(Whitespace).or_not())
        .repeated()
        .then_ignore(token(Newline))
        .node(Node::Block)
}
