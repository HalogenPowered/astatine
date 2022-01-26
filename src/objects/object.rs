use std::cell::RefCell;
use std::rc::Rc;
use crate::types::class::Class;
use crate::utils::vm_types::ArrayType;

/// Represents something that has been allocated on the heap by the VM, such as an object
/// or an array.
pub trait HeapObject {
    fn offset(&self) -> usize;

    fn class(&self) -> &Class;

    fn equals(&self, other: &Self) -> bool {
        self as *const Self == other as *const Self
    }
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

        fn get(&self, index: usize) -> u32 {
            self.$field_name.borrow().get(index).map_or(0, |value| *value)
        }

        fn put(&self, index: usize, value: u32) {
            self.$field_name.borrow_mut().insert(index, value);
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
    class: Rc<Class>,
    fields: RefCell<Vec<u32>>
}

impl InstanceObject {
    pub fn new(offset: usize, class: Rc<Class>, field_count: usize) -> Self {
        InstanceObject {
            offset,
            class,
            fields: RefCell::new(Vec::with_capacity(field_count))
        }
    }

    impl_getter_setter!(fields);
}

impl_heap_object!(InstanceObject);

pub struct ReferenceArrayObject {
    offset: usize,
    class: Rc<Class>,
    element_class: Rc<Class>,
    elements: RefCell<Vec<Rc<InstanceObject>>>
}

impl ReferenceArrayObject {
    pub fn new(
        offset: usize,
        class: Rc<Class>,
        element_class: Rc<Class>,
        size: usize
    ) -> Self {
        ReferenceArrayObject {
            offset,
            class,
            element_class,
            elements: RefCell::new(Vec::with_capacity(size))
        }
    }

    pub fn element_class(&self) -> &Class {
        &self.element_class
    }

    pub fn len(&self) -> usize {
        self.elements.borrow().len()
    }

    pub fn get(&self, index: usize) -> Option<Rc<InstanceObject>> {
        self.elements.borrow().get(index).map(|value| Rc::clone(value))
    }

    #[allow(unused_must_use)]
    pub fn set(&self, index: usize, value: Rc<InstanceObject>) {
        self.elements.borrow_mut().insert(index, value);
    }
}

impl_heap_object!(ReferenceArrayObject);

pub struct TypeArrayObject {
    offset: usize,
    class: Rc<Class>,
    array_type: ArrayType,
    elements: RefCell<Vec<u32>>
}

impl TypeArrayObject {
    pub fn new(
        offset: usize,
        class: Rc<Class>,
        array_type: ArrayType,
        size: usize
    ) -> Self {
        TypeArrayObject { offset, class, array_type, elements: RefCell::new(Vec::with_capacity(size)) }
    }

    pub fn array_type(&self) -> ArrayType {
        self.array_type
    }

    pub fn len(&self) -> usize {
        self.elements.borrow().len()
    }

    impl_getter_setter!(elements);
}

impl_heap_object!(TypeArrayObject);
