mod object;
mod heap;
mod reference;
pub mod handles;

pub use object::{HeapObject, InstanceObject, ReferenceArrayObject, TypeArrayObject};
pub use heap::HeapSpace;
pub use reference::Reference;
