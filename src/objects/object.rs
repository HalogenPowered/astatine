use std::sync::{Arc, RwLock};
use crate::types::class::Class;
use crate::utils::vm_types::ArrayType;

/// Represents something that has been allocated on the heap by the VM, such as an object
/// or an array.
pub trait HeapObject {
    fn offset(&self) -> usize;

    fn equals(&self, other: &Self) -> bool {
        self as *const Self == other as *const Self
    }
}

pub trait ReferenceHeapObject: HeapObject {
    fn class(&self) -> Arc<Class>;
}

macro_rules! impl_getter_setter {
    ($field_name:ident) => {
        pub fn get_bool(&self, index: usize) -> bool {
            self.get(index) != 0
        }

        pub fn get_byte(&self, index: usize) -> i8 {
            self.get(index) as i8
        }

        pub fn get_char(&self, index: usize) -> char {
            char::from_u32(self.get(index))
                .expect(&format!("Invalid character at index {}!", index))
        }

        pub fn get_short(&self, index: usize) -> i16 {
            self.get(index) as i16
        }

        pub fn get_int(&self, index: usize) -> i32 {
            self.get(index) as i32
        }

        pub fn get_float(&self, index: usize) -> f32 {
            f32::from_bits(self.get(index))
        }

        pub fn get_long(&self, index: usize) -> i64 {
            let most = self.get(index) as u64;
            let least = self.get(index + 1) as u64;
            ((most << 32) | least) as i64
        }

        pub fn get_double(&self, index: usize) -> f64 {
            let most = self.get(index) as u64;
            let least = self.get(index + 1) as u64;
            f64::from_bits((most << 32) | least)
        }

        pub fn put_bool(&self, index: usize, value: bool) {
            self.put(index, value as u32);
        }

        pub fn put_byte(&self, index: usize, value: i8) {
            self.put(index, value as u32);
        }

        pub fn put_char(&self, index: usize, value: char) {
            self.put(index, value as u32);
        }

        pub fn put_short(&self, index: usize, value: i16) {
            self.put(index, value as u32);
        }

        pub fn put_int(&self, index: usize, value: i32) {
            self.put(index, value as u32);
        }

        pub fn put_float(&self, index: usize, value: f32) {
            self.put(index, value.to_bits());
        }

        pub fn put_long(&self, index: usize, value: i64) {
            self.put(index, (value >> 32) as u32);
            self.put(index + 1, value as u32);
        }

        pub fn put_double(&self, index: usize, value: f64) {
            let bits = value.to_bits();
            self.put(index, (bits >> 32) as u32);
            self.put(index + 1, bits as u32);
        }

        pub fn get(&self, index: usize) -> u32 {
            self.$field_name.read().unwrap().get(index).map_or(0, |value| *value)
        }

        pub fn put(&self, index: usize, value: u32) {
            self.$field_name.write().unwrap().insert(index, value);
        }
    }
}

macro_rules! impl_heap_object {
    ($T:ident) => {
        impl HeapObject for $T {
            fn offset(&self) -> usize {
                self.offset
            }
        }

        impl ReferenceHeapObject for $T {
            fn class(&self) -> Arc<Class> {
                Arc::clone(&self.class)
            }
        }
    }
}

// TODO: Look in to storing a pointer to the start of memory instead of using a vec, which should
//  offer greater performance and lower memory footprint.
pub struct InstanceObject {
    offset: usize,
    class: Arc<Class>,
    fields: RwLock<Vec<u32>>
}

impl InstanceObject {
    pub fn new(offset: usize, class: Arc<Class>, field_count: usize) -> Self {
        InstanceObject {
            offset,
            class,
            fields: RwLock::new(Vec::with_capacity(field_count))
        }
    }

    impl_getter_setter!(fields);
}

impl_heap_object!(InstanceObject);

// TODO: Look in to storing a pointer to the start of memory instead of using a vec, which should
//  offer greater performance and lower memory footprint.
pub struct ReferenceArrayObject {
    offset: usize,
    class: Arc<Class>,
    element_class: Arc<Class>,
    length: usize,
    elements: RwLock<Vec<Arc<InstanceObject>>>
}

impl ReferenceArrayObject {
    pub fn new(
        offset: usize,
        class: Arc<Class>,
        element_class: Arc<Class>,
        length: usize
    ) -> Self {
        ReferenceArrayObject {
            offset,
            class,
            element_class,
            length,
            elements: RwLock::new(Vec::with_capacity(length))
        }
    }

    pub fn element_class(&self) -> &Class {
        &self.element_class
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn get(&self, index: usize) -> Option<Arc<InstanceObject>> {
        self.elements.read().unwrap().get(index).map(|value| Arc::clone(value))
    }

    #[allow(unused_must_use)]
    pub fn set(&self, index: usize, value: Rc<InstanceObject>) {
        self.elements.write().unwrap().insert(index, value);
    }
}

impl_heap_object!(ReferenceArrayObject);

// TODO: Look in to storing a pointer to the start of memory instead of using a vec, which should
//  offer greater performance and lower memory footprint.
pub struct TypeArrayObject {
    offset: usize,
    array_type: ArrayType,
    length: usize,
    elements: RwLock<Vec<u32>>
}

impl TypeArrayObject {
    pub fn new(offset: usize, array_type: ArrayType, length: usize) -> Self {
        TypeArrayObject { offset, array_type, length, elements: RwLock::new(Vec::with_capacity(length)) }
    }

    pub fn array_type(&self) -> ArrayType {
        self.array_type
    }

    pub fn len(&self) -> usize {
        self.length
    }

    impl_getter_setter!(elements);
}

impl HeapObject for TypeArrayObject {
    fn offset(&self) -> usize {
        self.offset
    }
}
