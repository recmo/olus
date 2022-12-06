

trait Value: Sized {
    /// Convert a string literal to a value.
    fn from_string(string: &str) -> Option<Self>;

    /// Convert a number literal to a value.
    fn from_number(digits: &str) -> Option<Self>;
}

pub struct Environment<V: Value> {
    variables: HashMap<usize, V>,
}

pub struct Closure<V: Value> {
    environment: Environment<V>,
    body: Block,
}
