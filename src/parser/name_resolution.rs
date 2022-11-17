use super::syntax::{Argument, Block, Call, Def, Group, Identifier, Line, Proc, Root};
use owo_colors::{DynColors, OwoColorize};
use rowan::ast::AstNode;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution(HashMap<Identifier, Identifier>);

pub fn resolve(root: Root) -> Resolution {
    let mut resolver = Resolver {
        map:        vec![HashMap::new()],
        resolution: HashMap::new(),
    };
    resolver.visit_root(root);
    Resolution(resolver.resolution)
}

impl Resolution {
    pub fn resolve(&self, identifier: &Identifier) -> Option<&Identifier> {
        self.0.get(identifier)
    }
}

struct Resolver {
    map:        Vec<HashMap<String, Identifier>>,
    resolution: HashMap<Identifier, Identifier>,
}

impl Resolver {
    fn visit_root(&mut self, root: Root) {
        for line in root.lines() {
            self.visit_line(line);
        }
    }

    fn visit_line(&mut self, line: Line) {
        match line.clone() {
            Line::Def(def) => self.visit_def(def),
            Line::Call(call) => self.visit_call(call),
        };
    }

    fn visit_def(&mut self, def: Def) {
        // If it has a name, the name goes into the current scope.
        if let Some(name) = def.proc().name() {
            self.visit_bind(name);
        }
        // If it has a block, everything else goes into the block's scope.
        // If not, everything goes in the current scope.
        if def.block().is_some() {
            self.map.push(HashMap::new());
        }
        for parameter in def.proc().parameters() {
            self.visit_bind(parameter);
        }
        if let Some(call) = def.call() {
            self.visit_call(call);
        }
        if let Some(block) = def.block() {
            for line in block.lines() {
                self.visit_line(line);
            }
            self.map.pop();
        }
    }

    fn visit_call(&mut self, call: Call) {
        if call.block().is_some() {
            self.map.push(HashMap::new());
        }
        for argument in call.arguments() {
            self.visit_argument(argument);
        }
        if let Some(block) = call.block() {
            for line in block.lines() {
                self.visit_line(line);
            }
            self.map.pop();
        }
    }

    fn visit_argument(&mut self, argument: Argument) {
        match argument {
            Argument::Identifier(identifier) => self.visit_reference(identifier),
            Argument::Group(group) => self.visit_group(group),
            _ => {}
        }
    }

    fn visit_group(&mut self, group: Group) {
        if let Some(def) = group.def() {
            self.visit_def(def);
        }
        if let Some(call) = group.call() {
            self.visit_call(call);
        }
    }

    fn visit_bind(&mut self, identifier: Identifier) {
        self.map
            .last_mut()
            .unwrap()
            .insert(identifier.text().to_owned(), identifier.clone());
        self.resolution.insert(identifier.clone(), identifier);
    }

    fn visit_reference(&mut self, identifier: Identifier) {
        let bind = self
            .map
            .iter()
            .rev()
            .filter_map(|scope| scope.get(identifier.text()))
            .next();
        if let Some(bind) = bind {
            self.resolution.insert(identifier, bind.clone());
        }
    }
}
