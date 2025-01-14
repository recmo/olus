use {
    crate::parser::syntax::{Argument, Call, Def, Group, Identifier, Line, Root},
    rowan::{
        TextRange,
        ast::{AstNode, AstPtr, SyntaxNodePtr},
    },
    std::{collections::HashMap, mem::size_of},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub binders: Vec<usize>,
    refs:        HashMap<usize, usize>,
}

impl Resolution {
    #[must_use]
    pub fn resolve(root: &Root) -> Self {
        let mut resolver = Resolver {
            map: vec![HashMap::new()],
            unbound: vec![HashMap::new()],
            ..Default::default()
        };
        resolver.visit_root(root);

        let mut binders = vec![];
        resolver.resolution.values().for_each(|id| {
            if !binders.contains(id) {
                binders.push(*id);
            }
        });

        Self {
            binders,
            refs: resolver.resolution,
        }
    }

    pub fn binders<'a>(&'a self, root: &'a Root) -> impl Iterator<Item = Identifier> + 'a {
        self.binders
            .iter()
            .filter_map(|offset| root.identifier_at(*offset))
    }

    #[must_use]
    pub fn lookup(&self, identifier: &Identifier, root: &Root) -> Option<Identifier> {
        self.refs
            .get(&identifier.offset())
            .and_then(|offset| root.identifier_at(*offset))
    }
}

#[derive(Debug, Default)]
struct Resolver {
    map:        Vec<HashMap<String, usize>>,
    unbound:    Vec<HashMap<String, Vec<usize>>>,
    resolution: HashMap<usize, usize>,
}

/// Single pass name resolution.
///
/// It maintains a stack of current scopes, and a stack of unbound references.
impl Resolver {
    fn visit_root(&mut self, root: &Root) {
        for line in root.lines() {
            self.visit_line(&line);
        }
    }

    fn visit_line(&mut self, line: &Line) {
        match line {
            Line::Def(def) => self.visit_def(def),
            Line::Call(call) => self.visit_call(call),
        };
    }

    fn visit_def(&mut self, def: &Def) {
        // If it has a name, the name goes into the current scope.
        if let Some(name) = def.procedure().name() {
            self.visit_bind(&name);
        }
        // If it has a block, everything else goes into the block's scope.
        // If not, everything goes in the current scope.
        if def.block().is_some() {
            self.push_scope();
        }
        for parameter in def.procedure().parameters() {
            self.visit_bind(&parameter);
        }
        if let Some(call) = def.call() {
            self.visit_call(&call);
        }
        if let Some(block) = def.block() {
            for line in block.lines() {
                self.visit_line(&line);
            }
            self.pop_scope();
        }
    }

    fn visit_call(&mut self, call: &Call) {
        if call.block().is_some() {
            self.push_scope();
        }
        for argument in call.arguments() {
            self.visit_argument(&argument);
        }
        if let Some(block) = call.block() {
            for line in block.lines() {
                self.visit_line(&line);
            }
            self.pop_scope();
        }
    }

    fn visit_argument(&mut self, argument: &Argument) {
        match argument {
            Argument::Identifier(identifier) => self.visit_reference(identifier),
            Argument::Group(group) => self.visit_group(group),
            _ => {}
        }
    }

    fn visit_group(&mut self, group: &Group) {
        if let Some(def) = group.def() {
            self.visit_def(&def);
        }
        if let Some(call) = group.call() {
            self.visit_call(&call);
        }
    }

    fn visit_bind(&mut self, identifier: &Identifier) {
        self.map
            .last_mut()
            .unwrap()
            .insert(identifier.text().to_owned(), identifier.offset());
        self.resolution
            .insert(identifier.offset(), identifier.offset());

        // Resolve any unbound references to this identifier.
        if let Some(unbound) = self.unbound.last_mut().unwrap().get_mut(identifier.text()) {
            for reference in unbound.drain(..) {
                self.resolution.insert(reference, identifier.offset());
            }
        }
    }

    fn visit_reference(&mut self, identifier: &Identifier) {
        let bind = self.map.last().unwrap().get(identifier.text());
        if let Some(bind) = bind {
            self.resolution.insert(identifier.offset(), *bind);
        } else {
            self.push_unbound(identifier);
        }
    }

    fn push_unbound(&mut self, identifier: &Identifier) {
        let map = self.unbound.last_mut().unwrap();
        let vec = map.entry(identifier.text().to_owned()).or_default();
        vec.push(identifier.offset());
    }

    fn push_scope(&mut self) {
        self.map.push(HashMap::new());
        self.unbound.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.map.pop();
        let current_bind = self.map.last().unwrap();

        // Move all unbound identifiers from the popped scope to the current scope.
        let popped = self.unbound.pop().unwrap();
        let current = self.unbound.last_mut().unwrap();
        for (text, identifiers) in popped {
            // Check if they are bound in the current scope.
            if let Some(bind) = current_bind.get(&text) {
                for reference in identifiers {
                    self.resolution.insert(reference, *bind);
                }
            } else {
                let vec = current.entry(text).or_default();
                vec.extend(identifiers);
            }
        }
    }
}
