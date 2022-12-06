//! Gives every variable a globally unique name.
use super::Resolution;
use crate::parser::syntax::Identifier;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Naming(HashMap<Identifier, String>);

impl Naming {
    #[must_use]
    pub fn name(resolution: &Resolution) -> Self {
        todo!()

        // let mut counts = HashMap::<&str, usize>::new();
        // for name in resolution.binders.iter() {
        //     counts
        //         .entry(name.text())
        //         .and_modify(|e| *e += 1)
        //         .or_insert(1);
        // }

        // let mut names = HashMap::new();
        // let mut running = HashMap::<&str, usize>::new();
        // for name in resolution.binds.iter() {
        //     let index = running
        //         .entry(name.text())
        //         .and_modify(|e| *e += 1)
        //         .or_insert(1);
        //     let count = counts.get(name.text()).unwrap();

        //     names.insert(
        //         name.clone(),
        //         format!("{}{}", name.text(), suffix(*index, *count)),
        //     );
        // }
        // Self(names)
    }

    #[must_use]
    pub fn lookup(&self, identifier: &Identifier) -> Option<&String> {
        self.0.get(identifier)
    }
}

fn suffix(index: usize, count: usize) -> String {
    const DIGITS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
    if count <= 1 {
        return String::new();
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let n_digits = (count as f64).log10().ceil() as usize;
    let mut suffix = String::with_capacity(n_digits);
    for i in 0..n_digits {
        #[allow(clippy::cast_possible_truncation)]
        suffix.push(DIGITS[(index / 10usize.pow(i as u32)) % 10]);
    }
    suffix
}
