use std::sync::{Arc, RwLock};
use paste::paste;
use super::object::*;
use super::reference::Reference;

// TODO: Look in to some more low-level allocation here, to maximise performance, minimise footprint,
//  allow lookups that are not thread-safe, and also so we can actually reserve the memory in
//  advanced so that no other processes can use our memory.
pub struct HeapSpace {
    allocated: RwLock<Vec<HeapEntry>>,
    maximum_size: usize
}

macro_rules! ref_get_push {
    ($name:ident, $type:ty, $entry:ident) => {
        paste! {
            pub fn [<get_ $name>](&self, index: usize) -> Reference<$type> {
                match self.allocated.read().unwrap().get(index) {
                    Some(HeapEntry::$entry(object)) => Reference::Value(Arc::clone(object)),
                    _ => Reference::Null
                }
            }

            pub fn [<push_ $name>](&self, object: Arc<$type>) {
                self.allocated.write().unwrap().push(HeapEntry::$entry(object))
            }
        }
    }
}

impl HeapSpace {
    pub fn new(maximum_size: usize) -> Self {
        HeapSpace { allocated: RwLock::new(Vec::new()), maximum_size }
    }

    pub fn len(&self) -> usize {
        self.allocated.read().unwrap().len()
    }

    ref_get_push!(ref, InstanceObject, Instance);
    ref_get_push!(ref_array, ReferenceArrayObject, ReferenceArray);
    ref_get_push!(type_array, TypeArrayObject, TypeArray);
}

enum HeapEntry {
    Instance(Arc<InstanceObject>),
    ReferenceArray(Arc<ReferenceArrayObject>),
    TypeArray(Arc<TypeArrayObject>)
}
