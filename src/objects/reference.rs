#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum Reference<T> {
    Value(T),
    Null
}

impl<T> Reference<T> {
    pub fn expect(self, message: &str) -> T {
        match self {
            Reference::Value(value) => value,
            Reference::Null => panic!("{}", message)
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            Reference::Value(value) => value,
            Reference::Null => panic!("called `Reference::unwrap()` on a `Null` value"),
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
