use olus::{front::Resolution, parser::parse};
use std::{fs::read_to_string, io::stdout};

fn main() {
    let source = read_to_string("./examples/test.olus").unwrap();
    let root = parse(&source).root();
    let resolution = Resolution::resolve(root.clone());
    olus::front::unparse(&mut stdout(), root.clone(), Some(&resolution)).unwrap();
    let naming = olus::front::Naming::name(&resolution);

    println!("-------------------");

    olus::front::list_defs(root);
}
