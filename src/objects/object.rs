use std::sync::RwLock;
use crate::types::class::Class;
use crate::utils::vm_types::ArrayType;

/// Represents something that has been allocated on the heap by the VM, such as an object
/// or an array.
pub trait HeapObject {
    fn offset(&self) -> usize;

    fn class(&self) -> &Class;
}

macro_rules! impl_getter_setter {
    ($field_name:ident) => {
        pub fn get_bool(&self, index: usize) -> bool {
            self.get(index).map_or(false, |value| *value != 0)
        }

        pub fn get_byte(&self, index: usize) -> i8 {
            self.get(index).map_or(0, |value| (*value & 255) as i8)
        }

        pub fn get_char(&self, index: usize) -> char {
            self.get(index).and_then(|value| char::from_u32(*value))
                .expect(&format!("Invalid character at index {}!", index))
        }

        pub fn get_short(&self, index: usize) -> i16 {
            self.get(index).map_or(0, |value| (*value & 65535) as i16)
        }

        pub fn get_int(&self, index: usize) -> i32 {
            self.get(index).map_or(0, |value| *value as i32)
        }

        pub fn get_float(&self, index: usize) -> f32 {
            self.get(index).map_or(0, |value| f32::from_bits(*value))
        }

        pub fn get_long(&self, index: usize) -> i64 {
            let most = self.get(index)?;
            let least = self.get(index + 1)?;
            (((*most as u64) << 32) | (*least as u64)) as i64
        }

        pub fn get_double(&self, index: usize) -> f64 {
            let most = self.get(index)?;
            let least = self.get(index + 1)?;
            f64::from_bits(((*most as u64) << 32) | (*least as u64))
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

        fn get(&self, index: usize) -> Option<&u32> {
            self.$field_name.read().ok().and_then(|read| read.get(index))
        }

        fn put(&self, index: usize, value: u32) {
            self.$field_name.write().map(|mut vector| vector.insert(index, value));
        }
    }
}

macro_rules! impl_heap_object {
    ($T:ident) => {
        impl HeapObject for $T {
            fn offset(&self) -> usize {
                self.offset
            }

            fn class(&self) -> &Class {
                unsafe { self.class.as_ref().unwrap() }
            }
        }
    }
}

pub struct InstanceObject {
    offset: usize,
    class: *const Class,
    fields: RwLock<Vec<u32>>
}

impl InstanceObject {
    pub fn new(offset: usize, class: &Class, field_count: usize) -> InstanceObject {
        InstanceObject {
            offset,
            class: class as *const Class,
            fields: RwLock::new(Vec::with_capacity(field_count))
        }
    }

    pub fn fields(&self) -> Option<&[u32]> {
        self.fields.read().ok().map(|vector| vector.as_slice())
    }

    impl_getter_setter!(fields);
}

impl_heap_object!(InstanceObject);

pub struct ReferenceArrayObject {
    offset: usize,
    class: *const Class,
    element_class: *const Class,
    elements: RwLock<Vec<*const Box<InstanceObject>>>
}

impl ReferenceArrayObject {
    pub fn new(
        offset: usize,
        class: &Class,
        element_class: &Class,
        size: usize
    ) -> ReferenceArrayObject {
        ReferenceArrayObject {
            offset,
            class: class as *const Class,
            element_class: element_class as *const Class,
            elements: RwLock::new(Vec::with_capacity(size))
        }
    }

    pub fn element_class(&self) -> &Class {
        unsafe { self.element_class.as_ref().unwrap() }
    }

    pub fn len(&self) -> usize {
        self.elements.read().map_or(0, |vector| vector.len())
    }

    pub fn get(&self, index: usize) -> Option<&Box<InstanceObject>> {
        unsafe {
            self.elements
                .read()
                .ok()
                .and_then(|vector| vector.get(index).and_then(|value| value.as_ref()))
        }
    }

    #[allow(unused_must_use)]
    pub fn set(&self, index: usize, value: &Box<InstanceObject>) {
        self.elements.write().map(|mut vector| vector.insert(index, value));
    }
}

impl_heap_object!(ReferenceArrayObject);

pub struct TypeArrayObject {
    offset: usize,
    class: *const Class,
    array_type: ArrayType,
    elements: RwLock<Vec<u32>>
}

impl TypeArrayObject {
    pub fn new(
        offset: usize,
        class: &Class,
        array_type: ArrayType,
        size: usize
    ) -> TypeArrayObject {
        TypeArrayObject { offset, class, array_type, elements: RwLock::new(Vec::with_capacity(size)) }
    }

    pub fn array_type(&self) -> ArrayType {
        self.array_type
    }

    pub fn elements(&self) -> Option<&[u32]> {
        self.elements.read().ok().map(|value| value.as_slice())
    }

    pub fn len(&self) -> usize {
        self.elements.read().map_or(0, |value| value.len())
    }

    impl_getter_setter!(elements);
}

impl_heap_object!(TypeArrayObject);
