use ariadne::{Label, Report, ReportKind, Source};
use olus::parser::parse;
use rowan::ast::AstNode;
use std::{
    fs::read_to_string,
    io::{stdout, Error, Write},
};

fn main() {
    let source = read_to_string("./examples/test.olus").unwrap();
    let root = parse(&source).root();
    olus::parser::print(root.clone());
    let resolution = olus::parser::resolve(root.clone());
    olus::parser::unparse(&mut stdout(), root.clone(), Some(resolution));
}
