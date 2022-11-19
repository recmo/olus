mod name_resolution;
mod renamer;
mod unparser;

use crate::parser::syntax::Root;

pub use self::{name_resolution::Resolution, renamer::Naming, unparser::unparse};

pub struct Analysis {
    pub root:       crate::parser::syntax::Root,
    pub resolution: Resolution,
    pub naming:     Naming,
}

pub fn list_defs(root: Root) {
    let resolution = Resolution::resolve(root.clone());

    for line in root.lines() {
        println!("{:?}", line);
        if let Some(def) = line.def() {
            println!("  {:?}", def);
        }
    }
}

pub fn visit_root(root: Root) {}
