use crate::objects::object::{HeapEntry, InstanceObject, ReferenceArrayObject, TypeArrayObject};

pub struct HeapSpace {
    allocated: Vec<HeapEntry>
}

impl HeapSpace {
    pub fn new() -> HeapSpace {
        HeapSpace { allocated: Vec::new() }
    }

    pub fn get_ref(&self, index: usize) -> Option<&Box<InstanceObject>> {
        match self.get(index) {
            Some(HeapEntry::Instance(object)) => Some(object),
            _ => None
        }
    }

    pub fn get_ref_array(&self, index: usize) -> Option<&Box<ReferenceArrayObject>> {
        match self.get(index) {
            Some(HeapEntry::ReferenceArray(array)) => Some(array),
            _ => None
        }
    }

    pub fn get_type_array(&self, index: usize) -> Option<&Box<TypeArrayObject>> {
        match self.get(index) {
            Some(HeapEntry::TypeArray(array)) => Some(array),
            _ => None
        }
    }

    fn get(&self, index: usize) -> Option<&HeapEntry> {
        self.allocated.get(index)
    }

    pub fn push_ref(&mut self, object: Box<InstanceObject>) {
        self.push(HeapEntry::Instance(object));
    }

    pub fn push_ref_array(&mut self, array: Box<ReferenceArrayObject>) {
        self.push(HeapEntry::ReferenceArray(array));
    }

    pub fn push_type_array(&mut self, array: Box<TypeArrayObject>) {
        self.push(HeapEntry::TypeArray(array));
    }

    fn push(&mut self, entry: HeapEntry) {
        self.allocated.push(entry);
    }

    pub fn get_offset(&self) -> usize {
        self.allocated.as_ptr() as usize
    }
}
