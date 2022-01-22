#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum Reference<T> {
    Value(T),
    Null
}

impl<T> Reference<T> {
    pub fn expect(self, message: &str) -> T {
        match self {
            Reference::Value(value) => value,
            _ => panic!("{}", message)
        }
    }
}

impl<T> From<Option<T>> for Reference<T> {
    fn from(option: Option<T>) -> Self {
        match option {
            Some(value) => Reference::Value(value),
            None => Reference::Null
        }
    }
}
