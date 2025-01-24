//! Intermediate Representation

use {
    crate::front::Span,
    petgraph::{
        algo::{condensation, toposort},
        graph::{DiGraph, NodeIndex},
    },
    std::mem::swap,
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

    /// Check if closures are valid.
    pub fn closure_check(&self) {
        for proc in &self.procedures {
            let contains = |id: u32| {
                proc.arguments.iter().any(|arg| arg.id == id) || proc.closure.contains(&id)
            };
            for atom in &proc.body {
                let Atom::Reference { id, .. } = atom else {
                    continue;
                };
                if let Some(closure) = self.procedures.iter().find(|p| p.id() == *id) {
                    for id in &closure.closure {
                        if !contains(*id) {
                            panic!("Procedure does not close over {id}.");
                        }
                    }
                } else {
                    if !contains(*id) {
                        panic!("Procedure references unknown procedure {id}.");
                    }
                }
            }
        }
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

            // Iterate on the component until convergence.
            let mut converged = false;
            while !converged {
                converged = true;
                for proc_index in component {
                    let proc = &self.procedures[*proc_index];
                    let contains = |id: u32| {
                        proc.arguments.iter().any(|arg| arg.id == id) || proc.closure.contains(&id)
                    };
                    let mut to_add = vec![];
                    for atom in &proc.body {
                        let Atom::Reference { id, .. } = atom else {
                            continue;
                        };
                        if contains(*id) {
                            continue;
                        }
                        if let Some(c) = self.procedures.iter().find(|p| p.id() == *id) {
                            for item in &c.closure {
                                if !contains(*item) {
                                    to_add.push(*item);
                                }
                            }
                        } else {
                            to_add.push(*id);
                        }
                    }
                    if !to_add.is_empty() {
                        converged = false;
                        self.procedures[*proc_index].closure.extend(&to_add);
                    }
                }
            }
        }
    }

    /// Remove unreachable procedures.
    pub fn tree_shake(&mut self, root: u32) {
        let mut live = vec![false; self.procedures.len()];
        let root_ix = self
            .procedures
            .iter()
            .position(|p| p.id() == root)
            .expect("Root not found.");
        let mut stack = vec![root_ix];
        while let Some(ix) = stack.pop() {
            if live[ix] {
                continue;
            }
            live[ix] = true;
            for atom in &self.procedures[ix].body {
                let Atom::Reference { id, .. } = atom else {
                    continue;
                };
                if let Some(j) = self.procedures.iter().position(|p| p.id() == *id) {
                    stack.push(j);
                }
            }
        }
        let mut procedures = vec![];
        swap(&mut procedures, &mut self.procedures);
        self.procedures = procedures
            .into_iter()
            .enumerate()
            .filter(|(i, _)| live[*i])
            .map(|(_, p)| p)
            .collect();
    }

    /// Collapse duplicate procedures.
    pub fn deduplicate(&mut self) {
        todo!()
    }
}

impl<B: Clone> Program<B> {
    /// Inlining of procedures.
    /// Any procedure that calls a known procedure can be inlined.
    pub fn inline(&mut self) {
        for i in 0..self.procedures.len() {
            let mut body = vec![];
            swap(&mut self.procedures[i].body, &mut body);
            // Repeatedly inline the call in the body.
            // TODO: Abort on cycles.
            loop {
                let Some(Atom::Reference { id, .. }) = body.first() else {
                    break;
                };
                let Some(call) = self.procedures.iter().find(|p| p.id() == *id) else {
                    break;
                };
                // Inline the known call.
                let mut new_body = call
                    .body
                    .iter()
                    .map(|atom| {
                        let Atom::Reference { id, .. } = atom else {
                            return atom.clone();
                        };
                        // Map the arguments to the call.
                        if let Some(j) = call.arguments.iter().position(|arg| arg.id == *id) {
                            return body[j].clone();
                        }
                        // TODO: If atom creates a closure, we need to argument-map the closure.
                        if let Some(j) = self.procedure_by_id(*id) {
                            todo!()
                        }
                        atom.clone()
                    })
                    .collect::<Vec<_>>();
                swap(&mut body, &mut new_body);
            }
            swap(&mut self.procedures[i].body, &mut body);
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
