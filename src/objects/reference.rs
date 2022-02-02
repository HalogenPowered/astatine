use std::sync::Arc;
use super::object::HeapObject;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum Reference<T: HeapObject> {
    Value(Arc<T>),
    Null
}

impl<T: HeapObject> Reference<T> {
    pub fn expect(self, message: &str) -> Arc<T> {
        match self {
            Reference::Value(value) => value,
            Reference::Null => panic!("{}", message)
        }
    }

    pub fn unwrap(self) -> Arc<T> {
        match self {
            Reference::Value(value) => value,
            Reference::Null => panic!("called `Reference::unwrap()` on a `Null` value"),
        }
    }

    pub fn is_not_null(&self) -> bool {
        matches!(self, Reference::Value(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Reference::Null)
    }

    pub fn equals(self, other: Self) -> bool {
        if self.is_null() && other.is_null() {
            return true;
        }
        if self.is_not_null() && other.is_not_null() {
            return self.unwrap().equals(&other.unwrap())
        }
        return false;
    }
}

impl<T: HeapObject> From<Option<Arc<T>>> for Reference<T> {
    fn from(option: Option<Arc<T>>) -> Self {
        match option {
            Some(value) => Reference::Value(value),
            None => Reference::Null
        }
    }
}
