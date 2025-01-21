//! Intermediate Representation

use {
    crate::front::Span,
    petgraph::{
        algo::{condensation, toposort},
        graph::{DiGraph, NodeIndex},
    },
};

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
    pub closure:   Vec<u32>,
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

impl<B> Procedure<B> {
    #[must_use]
    pub fn name(&self) -> &Identifier {
        self.arguments
            .first()
            .expect("ICE: Every procedure must have name as first argument.")
    }

    #[must_use]
    pub fn id(&self) -> u32 {
        self.name().id
    }
}

impl<B> Program<B> {
    #[must_use]
    pub fn procedure_by_id(&self, id: u32) -> Option<&Procedure<B>> {
        self.procedures.iter().find(|p| p.id() == id)
    }

    #[must_use]
    pub fn identifiers(&self) -> impl Iterator<Item = &Identifier> {
        self.procedures
            .iter()
            .flat_map(|proc| proc.arguments.iter())
    }

    #[must_use]
    pub fn string(&self, span: Span) -> &str {
        &self.source[span.start..span.end]
    }

    #[must_use]
    pub fn id_string(&self, id: u32) -> Option<&str> {
        self.identifiers().find(|i| i.id == id).and_then(|i| {
            if i.named {
                Some(self.string(i.source))
            } else {
                None
            }
        })
    }

    #[must_use]
    pub fn procedure_by_name(&self, name: &str) -> Option<&Procedure<B>> {
        self.procedures
            .iter()
            .find(|p| self.string(p.name().source) == name)
    }

    /// Construct a graph of closure dependencies.
    #[must_use]
    pub fn closure_graph(&self) -> DiGraph<usize, ()> {
        let mut graph = DiGraph::with_capacity(self.procedures.len(), 0);
        for i in 0..self.procedures.len() {
            graph.add_node(i);
        }
        for (i, proc) in self.procedures.iter().enumerate() {
            let from = NodeIndex::from(i as u32);
            for atom in &proc.body {
                // If it references a procedure name, we need to add a closure dependency.
                if let Atom::Reference { id, .. } = atom {
                    if let Some(j) = self.procedures.iter().position(|p| p.id() == *id) {
                        let to = NodeIndex::from(j as u32);
                        graph.add_edge(from, to, ());
                    }
                }
            }
        }
        graph
    }

    /// Perform closure analysis on the program.
    pub fn closure_analysis(&mut self) {
        // Solve recursion.
        // Iterate over the closure graph in condensed topological order. This means to
        // consider strongly connected components together, and work backwards
        // from visited nodes.
        let graph = self.closure_graph();
        let condensed = condensation(graph, true);
        let order = toposort(&condensed, None).expect("Condensation is acyclic.");
        for node in order.iter().rev() {
            let component = condensed.node_weight(*node).unwrap();

            // Collect all outgoing references
            let mut union = vec![];
            for proc_index in component {
                let proc = &self.procedures[*proc_index];
                for atom in &proc.body {
                    let Atom::Reference { id, .. } = atom else {
                        continue;
                    };
                    if union.contains(id) {
                        continue;
                    }
                    if proc.arguments.iter().any(|arg| arg.id == *id) {
                        continue;
                    }
                    if let Some(proc) = self.procedures.iter().find(|p| p.id() == *id) {
                        for item in &proc.closure {
                            if !union.contains(item) {
                                union.push(*item);
                            }
                        }
                    } else {
                        union.push(*id);
                    }
                }
            }

            // Store the closures, ignoring any brought in by their arguments.
            for proc_index in component {
                let proc = &mut self.procedures[*proc_index];
                proc.closure = union
                    .iter()
                    .copied()
                    .filter(|&id| proc.arguments.iter().all(|arg| arg.id != id))
                    .collect();
            }
        }
    }
}

pub fn pretty_print_ir<B>(program: &Program<B>) {
    for proc in &program.procedures {
        for (i, arg) in proc.arguments.iter().enumerate() {
            if let Some(name) = program.id_string(arg.id) {
                eprint!("{name}_{}", arg.id);
            } else {
                eprint!("_{}", arg.id);
            }
            if i != proc.arguments.len() - 1 {
                eprint!(" ");
            }
        }
        if !proc.closure.is_empty() {
            eprint!("; ");
            for (i, arg) in proc.closure.iter().enumerate() {
                if let Some(name) = program.id_string(*arg) {
                    eprint!("{name}_{}", *arg);
                } else {
                    eprint!("_{}", *arg);
                }
                if i != proc.closure.len() - 1 {
                    eprint!(" ");
                }
            }
        }
        eprint!(":");
        for a in &proc.body {
            eprint!(" ");
            match a {
                Atom::Builtin { source, .. } => {
                    eprint!("@{}", program.string(*source));
                }
                Atom::Number { value, .. } => eprint!("{value}"),
                Atom::String { value, .. } => eprint!("{value:?}"),
                Atom::Reference { id, .. } => {
                    if let Some(name) = program.id_string(*id) {
                        eprint!("{name}_{id}");
                    } else {
                        eprint!("_{id}");
                    }
                }
            }
        }
        eprintln!();
    }
}
