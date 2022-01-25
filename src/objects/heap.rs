use std::rc::Rc;
use paste::paste;
use super::object::*;
use super::reference::Reference;

pub struct HeapSpace<'a> {
    allocated: Vec<HeapEntry<'a>>
}

macro_rules! ref_get_push {
    ($name:ident, $type:ty, $entry:ident) => {
        paste! {
            pub fn [<get_ $name>](&self, index: usize) -> Reference<Rc<$type>> {
                match self.allocated.get(index) {
                    Some(HeapEntry::$entry(object)) => Reference::Value(Rc::clone(object)),
                    _ => Reference::Null
                }
            }

            pub fn [<push_ $name>](&mut self, object: Rc<$type>) {
                self.allocated.push(HeapEntry::$entry(object))
            }
        }
    }
}

impl<'a> HeapSpace<'a> {
    pub const fn new() -> Self {
        HeapSpace { allocated: Vec::new() }
    }

    pub fn get_offset(&self) -> usize {
        self.allocated.as_ptr() as usize
    }

    ref_get_push!(ref, InstanceObject<'a>, Instance);
    ref_get_push!(ref_array, ReferenceArrayObject<'a>, ReferenceArray);
    ref_get_push!(type_array, TypeArrayObject<'a>, TypeArray);
}

enum HeapEntry<'a> {
    Instance(Rc<InstanceObject<'a>>),
    ReferenceArray(Rc<ReferenceArrayObject<'a>>),
    TypeArray(Rc<TypeArrayObject<'a>>)
}
