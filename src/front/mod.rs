mod name_resolution;
mod renamer;
mod unparser;

use crate::{
    files::{FileId, Files},
    parser::syntax::{Argument, Identifier, Line, Root},
};
use ariadne::{Color, Label, Report, ReportKind, Source};
use rowan::ast::AstNode;
use std::{
    fmt::{Display, Write},
    ops::Range,
};

pub use self::{name_resolution::Resolution, renamer::Naming, unparser::unparse};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sugared<'a> {
    source:      &'a str,
    definitions: Vec<Definition>,
    binders:     Vec<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Definition {
    procedure: Vec<usize>,
    call:      Vec<Expression>,
    span:      Range<usize>,
    call_span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Reference {
        binder: usize,
        span:   Range<usize>,
    },
    String {
        span: Range<usize>,
    },
    Number {
        value: u64,
        span:  Range<usize>,
    },
    Definition {
        span:      Range<usize>,
        procedure: Vec<usize>,
        call:      Vec<Expression>,
    },
    Call {
        span: Range<usize>,
        call: Vec<Expression>,
    },
}

impl<'a> Display for Sugared<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write(f)
    }
}

impl<'a> Sugared<'a> {
    fn write<W: Write>(&self, f: &mut W) -> std::fmt::Result {
        for definition in self.definitions.iter() {
            self.write_definition(f, &definition)?;
            writeln!(f)?;
        }
        Ok(())
    }

    fn write_identifier<W: Write>(&self, f: &mut W, id: usize) -> std::fmt::Result {
        write!(f, "{}", &self.source[self.binders[id].clone()])
    }

    fn write_definition<W: Write>(&self, f: &mut W, definition: &Definition) -> std::fmt::Result {
        for (i, param) in definition.procedure.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            self.write_identifier(f, *param)?;
        }
        write!(f, ": ")?;
        for (i, arg) in definition.call.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            self.write_expression(f, arg)?;
        }
        Ok(())
    }

    fn write_expression<W: Write>(&self, f: &mut W, expression: &Expression) -> std::fmt::Result {
        match expression {
            Expression::Reference { binder, .. } => self.write_identifier(f, *binder),
            Expression::String { span } => write!(f, "“{}”", &self.source[span.clone()]),
            Expression::Number { value, .. } => write!(f, "{}", value),
            Expression::Definition {
                procedure, call, ..
            } => {
                for (i, param) in procedure.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    self.write_identifier(f, *param)?;
                }
                write!(f, ": ")?;
                for (i, arg) in call.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    self.write_expression(f, arg)?;
                }
                Ok(())
            }
            Expression::Call { span, call } => {
                write!(f, "(")?;
                for (i, arg) in call.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    self.write_expression(f, arg)?;
                }
                write!(f, ")")
            }
        }
    }
}

pub fn parse<'a>(files: &'a Files, file_id: FileId) -> Sugared<'a> {
    let source = files[file_id].contents();
    let parse = crate::parser::parse(file_id, source);

    for error in &parse.errors {
        error.report().eprint(files);
    }
    if !parse.errors.is_empty() {
        panic!("Parse errors");
    }

    let root = parse.root();
    let mut parser = Parser {
        sugared:    Sugared {
            source,
            definitions: Vec::new(),
            binders: Vec::new(),
        },
        root:       root.clone(),
        resolution: Resolution::resolve(root.clone()),
    };
    parser.collect_binders();
    parser.parse_root();
    parser.sugared
}

struct Parser<'a> {
    sugared:    Sugared<'a>,
    root:       Root,
    resolution: Resolution,
}

impl<'a> Parser<'a> {
    fn collect_binders(&mut self) {
        for binder in self.resolution.binders(&self.root) {
            self.sugared
                .binders
                .push(binder.syntax().text_range().into());
        }
    }

    fn lookup_binder(&self, id: &Identifier) -> Option<usize> {
        let id = self.resolution.lookup(id, &self.root)?;
        let offset = id.offset();
        self.sugared
            .binders
            .iter()
            .position(|range| range.start == offset)
    }

    fn parse_root(&mut self) {
        for def in self.root.defs() {
            // Add the identifiers from the procedure
            let mut procedure = Vec::new();
            for id in def.procedure().identifiers() {
                procedure.push(self.lookup_binder(&id).unwrap());
            }

            // Construct the expression
            let mut expression = def
                .call()
                .unwrap()
                .arguments()
                .map(|arg| self.parse_argument(arg))
                .collect::<Vec<_>>();

            // Add the definition
            self.sugared.definitions.push(Definition {
                procedure,
                call: expression,
                span: def.syntax().text_range().into(),
                call_span: def.call().unwrap().syntax().text_range().into(),
            });
        }
    }

    fn parse_argument(&mut self, arg: Argument) -> Expression {
        match arg {
            Argument::Identifier(id) => {
                let binder = self.resolution.lookup(&id, &self.root).unwrap().offset();
                let binder = self.sugared.binders.iter().position(|b| b.start == binder);
                let Some(binder) = binder else {
                    panic!("Could not find binder for identifier");
                };
                let span = id.syntax().text_range().into();
                Expression::Reference { binder, span }
            }
            Argument::String(string) => Expression::String {
                span: string.syntax().text_range().into(),
            },
            Argument::Number(number) => Expression::Number {
                value: number.syntax().text().parse().unwrap(),
                span:  number.syntax().text_range().into(),
            },
            Argument::Group(group) => {
                if let Some(def) = group.def() {
                    let mut procedure = Vec::new();
                    for id in def.procedure().identifiers() {
                        procedure.push(self.lookup_binder(&id).unwrap());
                    }
                    dbg!(def.syntax().text());
                    let mut call = def
                        .call()
                        .unwrap()
                        .arguments()
                        .map(|arg| self.parse_argument(arg))
                        .collect::<Vec<_>>();

                    Expression::Definition {
                        span: def.syntax().text_range().into(),
                        procedure,
                        call,
                    }
                } else if let Some(call) = group.call() {
                    Expression::Call {
                        span: call.syntax().text_range().into(),
                        call: call
                            .arguments()
                            .map(|arg| self.parse_argument(arg))
                            .collect::<Vec<_>>(),
                    }
                } else {
                    unreachable!()
                }
            }
        }
    }
}
