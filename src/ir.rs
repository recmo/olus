//! Intermediate Representation

use crate::front::Span;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Identifier {
    pub source: Span,
    pub named:  bool,
    pub id:     u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Atom<B> {
    Builtin { source: Span, builtin: B },
    Reference { source: Span, id: u32 },
    String { source: Span, value: String },
    Number { source: Span, value: u64 },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Procedure<B> {
    pub source:    Span,
    pub arguments: Vec<Identifier>,
    pub body:      Vec<Atom<B>>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Program<B> {
    pub source:     String,
    pub procedures: Vec<Procedure<B>>,
}

impl<B> Atom<B> {
    #[must_use]
    pub const fn source(&self) -> Span {
        match self {
            Self::Builtin { source, .. }
            | Self::Reference { source, .. }
            | Self::String { source, .. }
            | Self::Number { source, .. } => *source,
        }
    }
}

impl<B> Program<B> {
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

pub fn pretty_print_ir<B>(program: &Program<B>) {
    for proc in &program.procedures {
        for (i, arg) in proc.arguments.iter().enumerate() {
            if let Some(name) = program.resolve_name(arg.id) {
                eprint!("{name}_{}", arg.id);
            } else {
                eprint!("_{}", arg.id);
            }
            if i != proc.arguments.len() - 1 {
                eprint!(" ");
            }
        }
        eprint!(":");
        for a in &proc.body {
            eprint!(" ");
            match a {
                Atom::Builtin { source, .. } => {
                    eprint!("@{}", &program.source[source.start..source.end]);
                }
                Atom::Number { value, .. } => eprint!("{value}"),
                Atom::String { value, .. } => eprint!("{value}"),
                Atom::Reference { id, .. } => {
                    if let Some(name) = program.resolve_name(*id) {
                        eprint!("{name}");
                    } else {
                        eprint!("_{id}");
                    }
                }
            }
        }
        eprintln!();
    }
}
