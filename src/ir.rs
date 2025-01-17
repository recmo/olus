//! Intermediate Representation

use crate::parser::Span;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Identifier {
    pub source: Span,
    pub id:     u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Atom {
    Reference { source: Span, id: u32 },
    String { source: Span, value: String },
    Number { source: Span, value: u64 },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Procedure {
    pub source:    Span,
    pub arguments: Vec<Identifier>,
    pub body:      Vec<Atom>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Program {
    pub procedures: Vec<Procedure>,
}

impl Atom {
    pub fn source(&self) -> Span {
        match self {
            Self::Reference { source, .. } => *source,
            Self::String { source, .. } => *source,
            Self::Number { source, .. } => *source,
        }
    }
}
