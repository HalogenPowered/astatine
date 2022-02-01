use std::sync::{Arc, RwLock};
use crate::types::class::Class;
use crate::utils::vm_types::ArrayType;

/// Represents something that has been allocated on the heap by the VM, such as an object
/// or an array.
pub trait HeapObject {
    fn offset(&self) -> usize;

    fn len(&self) -> usize;

    fn equals(&self, other: &Self) -> bool {
        self as *const Self == other as *const Self
    }
}

pub trait ReferenceHeapObject: HeapObject {
    fn class(&self) -> &Class;
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

        pub fn set_bool(&self, index: usize, value: bool) {
            self.set(index, value as u32);
        }

        pub fn set_byte(&self, index: usize, value: i8) {
            self.set(index, value as u32);
        }

        pub fn set_char(&self, index: usize, value: char) {
            self.set(index, value as u32);
        }

        pub fn set_short(&self, index: usize, value: i16) {
            self.set(index, value as u32);
        }

        pub fn set_int(&self, index: usize, value: i32) {
            self.set(index, value as u32);
        }

        pub fn set_float(&self, index: usize, value: f32) {
            self.set(index, value.to_bits());
        }

        pub fn set_long(&self, index: usize, value: i64) {
            self.set(index, (value >> 32) as u32);
            self.set(index + 1, value as u32);
        }

        pub fn set_double(&self, index: usize, value: f64) {
            let bits = value.to_bits();
            self.set(index, (bits >> 32) as u32);
            self.set(index + 1, bits as u32);
        }

        pub fn get(&self, index: usize) -> u32 {
            self.$field_name.read().unwrap().get(index).map_or(0, |value| *value)
        }

        pub fn set(&self, index: usize, value: u32) {
            assert!(index < self.len(), "Index {} out of bounds for length {}!", index, self.len());
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

            fn len(&self) -> usize {
                self.length
            }
        }

        impl ReferenceHeapObject for $T {
            fn class(&self) -> &Class {
                &self.class
            }
        }
    }
}

// TODO: Look in to storing a pointer to the start of memory instead of using a vec, which should
//  offer greater performance and lower memory footprint.
pub struct InstanceObject {
    offset: usize,
    class: Arc<Class>,
    length: usize,
    fields: RwLock<Vec<u32>>
}

impl InstanceObject {
    pub fn new(offset: usize, class: Arc<Class>, length: usize) -> Self {
        InstanceObject {
            offset,
            class,
            length,
            fields: RwLock::new(Vec::with_capacity(length))
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

    pub fn get(&self, index: usize) -> Option<Arc<InstanceObject>> {
        self.elements.read().unwrap().get(index).map(|value| Arc::clone(value))
    }

    #[allow(unused_must_use)]
    pub fn set(&self, index: usize, value: Arc<InstanceObject>) {
        assert!(index < self.length, "Index {} out of bounds for length {}!", index, self.length);
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

    impl_getter_setter!(elements);
}

impl HeapObject for TypeArrayObject {
    fn offset(&self) -> usize {
        self.offset
    }

    fn len(&self) -> usize {
        self.length
    }
}
