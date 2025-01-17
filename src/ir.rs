//! Intermediate Representation

use crate::front::Span;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Identifier {
    pub source: Span,
    pub named:  bool,
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
    pub source:     String,
    pub procedures: Vec<Procedure>,
}

impl Atom {
    #[must_use]
    pub const fn source(&self) -> Span {
        match self {
            Self::Reference { source, .. }
            | Self::String { source, .. }
            | Self::Number { source, .. } => *source,
        }
    }
}

impl Program {
    #[must_use]
    pub fn resolve_name(&self, id: u32) -> Option<&str> {
        for proc in &self.procedures {
            for arg in &proc.arguments {
                if arg.id == id && arg.named {
                    return Some(&self.source[arg.source.start..arg.source.end]);
                }
            }
        }
        None
    }
}
