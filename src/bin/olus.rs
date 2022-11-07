use ariadne::{Label, Report, ReportKind, Source};
use logos::Lexer;
use olus::lexer::Token;
use std::io;

fn main() {
    let source = std::fs::read_to_string("./examples/test.olus").unwrap();

    let mut lexer = Lexer::<Token>::new(&source);

    for (token, span) in lexer.spanned() {
        println!("{:?}: {:?}", token, &source[span]);
    }

    let node = olus::parser::parse(&source).syntax();
    println!("{:?}", node);
    println!("{:?}", node.children_with_tokens().count());

    for child in node.children_with_tokens() {
        println!("{:?}@{:?}", child.kind(), child.text_range());

        let range = usize::from(child.text_range().start())..child.text_range().end().into();
        let message = format!("Found a {:?}", child.kind());
        Report::build(ReportKind::Advice, (), 0)
            .with_message("Parsing")
            .with_label(Label::new(range).with_message(message))
            .finish()
            .print(Source::from(source.clone()))
            .unwrap();
    }
}
