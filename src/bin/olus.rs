use std::fs::read_to_string;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");

    let source = read_to_string("./examples/test.olus").unwrap();
    let sugared = olus::front::parse(&source);
    println!("{}", &sugared);
}
