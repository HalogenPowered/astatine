use std::rc::Rc;
use crate::objects::object::{HeapEntry, InstanceObject, ReferenceArrayObject, TypeArrayObject};

pub struct HeapSpace {
    allocated: Vec<Rc<HeapEntry>>
}

pub static HEAP_SPACE: HeapSpace = HeapSpace { allocated: Vec::new() };

impl HeapSpace {
    pub fn get_ref(&self, index: usize) -> Option<&InstanceObject> {
        match self.get(index) {
            Some(HeapEntry::Instance(object)) => Some(object),
            _ => None
        }
    }

    pub fn get_ref_array(&self, index: usize) -> Option<&ReferenceArrayObject> {
        match self.get(index) {
            Some(HeapEntry::ReferenceArray(array)) => Some(array),
            _ => None
        }
    }

    pub fn get_type_array(&self, index: usize) -> Option<&TypeArrayObject> {
        match self.get(index) {
            Some(HeapEntry::TypeArray(array)) => Some(array),
            _ => None
        }
    }

    fn get(&self, index: usize) -> Option<&HeapEntry> {
        self.allocated.get(index).map(|value| value.as_ref())
    }
}

unsafe impl Sync for HeapSpace {
}
