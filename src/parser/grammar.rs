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

    let call = expression
        .separated_by(token(Whitespace))
        .at_least(1)
        .then_ignore(token(Newline))
        .node(Call);

    let procedure = token(Identifier)
        .separated_by(token(Whitespace))
        .at_least(1)
        .then_ignore(token(Colon).padded_by(token(Whitespace).or_not()))
        .then_ignore(call.clone().or(token(Newline)))
        .node(Proc);

    let block = recursive(|block| {
        choice((
            procedure,
            call,
            block.delimited_by(token(Indent), token(Dedent)).node(Block),
        ))
        .repeated()
    });

    block
}
