use std::sync::RwLock;
use crate::objects::object::{HeapEntry, InstanceObject, ReferenceArrayObject, TypeArrayObject};
use crate::types::class::Class;
use crate::utils::vm_types::ArrayType;

pub struct HeapSpace<'a> {
    start_address: usize,
    allocated: RwLock<Vec<HeapEntry<'a>>>
}

impl<'a> HeapSpace<'a> {
    pub fn new(start_address: usize) -> HeapSpace<'a> {
        HeapSpace { start_address, allocated: RwLock::new(Vec::new()) }
    }

    pub fn get_ref(&self, index: usize) -> Option<&'a InstanceObject<'a>> {
        match self.get(index) {
            Some(HeapEntry::Instance(object)) => Some(object),
            _ => None
        }
    }

    pub fn get_ref_array(&self, index: usize) -> Option<&'a ReferenceArrayObject<'a, 'a>> {
        match self.get(index) {
            Some(HeapEntry::ReferenceArray(array)) => Some(array),
            _ => None
        }
    }

    pub fn get_type_array(&self, index: usize) -> Option<&'a TypeArrayObject<'a>> {
        match self.get(index) {
            Some(HeapEntry::TypeArray(array)) => Some(array),
            _ => None
        }
    }

    fn get(&self, index: usize) -> Option<&HeapEntry<'a>> {
        self.allocated.read().ok().and_then(|vec| vec.get(index))
    }

    pub fn push_ref(&self, class: &'a mut Class, field_count: usize) -> InstanceObject<'a> {
        let object = InstanceObject::new(self.get_offset(), class, field_count);
        self.push(HeapEntry::Instance(&object));
        object
    }

    pub fn push_ref_array(&self, class: &'a mut Class, element_class: &'a mut Class, size: usize) -> ReferenceArrayObject<'a, 'a> {
        let array = ReferenceArrayObject::new(self.get_offset(), class, element_class, size);
        self.push(HeapEntry::ReferenceArray(&array));
        array
    }

    pub fn push_type_array(&self, class: &'a mut Class, array_type: ArrayType, size: usize) -> TypeArrayObject<'a> {
        let array = TypeArrayObject::new(self.get_offset(), class, array_type, size);
        self.push(HeapEntry::TypeArray(&array));
        array
    }

    fn push(&self, entry: HeapEntry<'a>) {
        self.allocated.write().ok().map(|mut vec| vec.push(entry));
    }

    fn get_offset(&self) -> usize {
        self.allocated.read().ok().map(|value| value.as_ptr() as usize)
            .expect("Could not read pointer of heap allocation vector!")
    }
}
