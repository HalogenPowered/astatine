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
        pub fn get_bool(&self, index: usize) -> Option<bool> {
            self.$field_name.get(index).map(|value| *value != 0)
        }

        pub fn get_byte(&self, index: usize) -> Option<i8> {
            self.$field_name.get(index).map(|value| (*value & 255) as i8)
        }

        pub fn get_char(&self, index: usize) -> Option<char> {
            self.$field_name.get(index).and_then(|value| char::from_u32(*value))
        }

        pub fn get_short(&self, index: usize) -> Option<i16> {
            self.$field_name.get(index).map(|value| (*value & 65535) as i16)
        }

        pub fn get_int(&self, index: usize) -> Option<i32> {
            self.$field_name.get(index).map(|value| *value as i32)
        }

        pub fn get_float(&self, index: usize) -> Option<f32> {
            self.$field_name.get(index).map(|value| f32::from_bits(*value))
        }

        pub fn get_long(&self, index: usize) -> Option<i64> {
            let most = self.$field_name.get(index)?;
            let least = self.$field_name.get(index + 1)?;
            Some((((*most as u64) << 32) | (*least as u64)) as i64)
        }

        pub fn get_double(&self, index: usize) -> Option<f64> {
            let most = self.$field_name.get(index)?;
            let least = self.$field_name.get(index + 1)?;
            Some(f64::from_bits(((*most as u64) << 32) | (*least as u64)))
        }

        pub fn put_bool(&mut self, index: usize, value: bool) {
            self.$field_name.insert(index, value as u32);
        }

        pub fn put_byte(&mut self, index: usize, value: i8) {
            self.$field_name.insert(index, value as u32);
        }

        pub fn put_char(&mut self, index: usize, value: char) {
            self.$field_name.insert(index, value as u32);
        }

        pub fn put_short(&mut self, index: usize, value: i16) {
            self.$field_name.insert(index, value as u32);
        }

        pub fn put_int(&mut self, index: usize, value: i32) {
            self.$field_name.insert(index, value as u32);
        }

        pub fn put_float(&mut self, index: usize, value: f32) {
            self.$field_name.insert(index, value.to_bits());
        }

        pub fn put_long(&mut self, index: usize, value: i64) {
            self.$field_name.insert(index, (value >> 32) as u32);
            self.$field_name.insert(index + 1, value as u32);
        }

        pub fn put_double(&mut self, index: usize, value: f64) {
            let bits = value.to_bits();
            self.$field_name.insert(index, (bits >> 32) as u32);
            self.$field_name.insert(index + 1, bits as u32);
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
                &self.class
            }
        }
    }
}

pub struct InstanceObject {
    offset: usize,
    class: Box<Class>,
    fields: Vec<u32>
}

impl InstanceObject {
    pub fn new(offset: usize, class: Box<Class>, field_count: usize) -> InstanceObject {
        InstanceObject { offset, class, fields: Vec::with_capacity(field_count) }
    }

    pub fn fields(&self) -> &[u32] {
        self.fields.as_slice()
    }

    impl_getter_setter!(fields);
}

impl_heap_object!(InstanceObject);

pub struct ReferenceArrayObject {
    offset: usize,
    class: Box<Class>,
    element_class: Box<Class>,
    elements: Vec<*const Box<InstanceObject>>
}

impl ReferenceArrayObject {
    pub fn new(
        offset: usize,
        class: Box<Class>,
        element_class: Box<Class>,
        size: usize
    ) -> ReferenceArrayObject {
        ReferenceArrayObject { offset, class, element_class, elements: Vec::with_capacity(size) }
    }

    pub fn element_class(&self) -> &Class {
        &self.element_class
    }

    pub fn get(&self, index: usize) -> Option<&Box<InstanceObject>> {
        unsafe { self.elements.get(index).and_then(|value| value.as_ref()) }
    }

    pub fn set(&mut self, index: usize, value: &Box<InstanceObject>) {
        self.elements.insert(index, value);
    }
}

impl_heap_object!(ReferenceArrayObject);

pub struct TypeArrayObject {
    offset: usize,
    class: Box<Class>,
    array_type: ArrayType,
    elements: Vec<u32>
}

impl TypeArrayObject {
    pub fn new(
        offset: usize,
        class: Box<Class>,
        array_type: ArrayType,
        size: usize
    ) -> TypeArrayObject {
        TypeArrayObject { offset, class, array_type, elements: Vec::with_capacity(size) }
    }

    pub fn array_type(&self) -> ArrayType {
        self.array_type
    }

    pub fn elements(&self) -> &[u32] {
        self.elements.as_slice()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    impl_getter_setter!(elements);
}

impl_heap_object!(TypeArrayObject);
