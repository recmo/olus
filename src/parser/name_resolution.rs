use super::SyntaxNode;
use std::collections::HashMap;

pub struct Resolution {
    map: HashMap<usize, usize>,
}

impl Resolution {
    fn new(root: SyntaxNode) -> Resolution {
        Resolution {
            map: HashMap::new(),
        }
    }

    fn analyse(&mut self) {}

    fn insert(&mut self, key: usize, value: usize) {
        self.map.insert(key, value);
    }

    fn get(&self, key: usize) -> Option<usize> {
        self.map.get(&key).map(|x| *x)
    }
}
