use crate::objects::object::*;

pub struct HeapSpace {
    allocated: Vec<HeapEntry>
}

macro_rules! ref_get_push {
    ($get_name:ident, $push_name:ident, $type:ty, $entry:ident) => {
        pub fn $get_name(&self, index: usize) -> Option<&Box<$type>> {
            match self.allocated.get(index) {
                Some(HeapEntry::$entry(object)) => Some(object),
                _ => None
            }
        }

        pub fn $push_name(&mut self, object: Box<$type>) {
            self.allocated.push(HeapEntry::$entry(object))
        }
    }
}

impl HeapSpace {
    pub const fn new() -> HeapSpace {
        HeapSpace { allocated: Vec::new() }
    }

    pub fn get_offset(&self) -> usize {
        self.allocated.as_ptr() as usize
    }

    ref_get_push!(get_ref, push_ref, InstanceObject, Instance);
    ref_get_push!(get_ref_array, push_ref_array, ReferenceArrayObject, ReferenceArray);
    ref_get_push!(get_type_array, push_type_array, TypeArrayObject, TypeArray);
}

enum HeapEntry {
    Instance(Box<InstanceObject>),
    ReferenceArray(Box<ReferenceArrayObject>),
    TypeArray(Box<TypeArrayObject>)
}
