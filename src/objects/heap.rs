use std::rc::Rc;
use std::sync::Mutex;
use paste::paste;
use super::object::*;
use super::reference::Reference;

pub struct HeapSpace {
    allocated: Mutex<Vec<HeapEntry>>,
    maximum_size: usize
}

macro_rules! ref_get_push {
    ($name:ident, $type:ty, $entry:ident) => {
        paste! {
            pub fn [<get_ $name>](&self, index: usize) -> Reference<Rc<$type>> {
                match self.allocated.lock().unwrap().get(index) {
                    Some(HeapEntry::$entry(object)) => Reference::Value(Rc::clone(object)),
                    _ => Reference::Null
                }
            }

            pub fn [<push_ $name>](&self, object: Rc<$type>) {
                self.allocated.lock().unwrap().push(HeapEntry::$entry(object))
            }
        }
    }
}

impl HeapSpace {
    pub fn new(maximum_size: usize) -> Self {
        HeapSpace { allocated: Mutex::new(Vec::new()), maximum_size }
    }

    pub fn len(&self) -> usize {
        self.allocated.lock().unwrap().len()
    }

    ref_get_push!(ref, InstanceObject, Instance);
    ref_get_push!(ref_array, ReferenceArrayObject, ReferenceArray);
    ref_get_push!(type_array, TypeArrayObject, TypeArray);
}

enum HeapEntry {
    Instance(Rc<InstanceObject>),
    ReferenceArray(Rc<ReferenceArrayObject>),
    TypeArray(Rc<TypeArrayObject>)
}
