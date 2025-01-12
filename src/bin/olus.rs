use olus::Files;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // std::env::set_var("RUST_BACKTRACE", "full");
    let path = PathBuf::from("./examples/test.olus");

    let mut files = Files::new();
    let file_id = files.insert(path)?;

    let sugared = olus::front::parse(&files, file_id);
    println!("{}", &sugared);

    Ok(())
}
