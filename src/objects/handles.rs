use enum_as_inner::EnumAsInner;
use internship::IStr;
use std::fmt::Debug;
use std::sync::Arc;
use crate::class_file::ClassFileVersion;
use crate::types::{Class, ConstantPool};
use crate::utils::constants::{CLASS_INITIALIZER_METHOD_NAME, OBJECT_INITIALIZER_METHOD_NAME};
use crate::utils::descriptors::{FieldDescriptor, MethodDescriptor};

#[derive(Debug)]
pub struct MethodHandle {
    kind: u8,
    reference: MethodHandleRef
}

impl MethodHandle {
    pub(crate) fn parse(
        pool: &ConstantPool,
        kind: u8,
        reference_index: u16,
        version: &ClassFileVersion
    ) -> Self {
        assert!(kind >= REF_GET_FIELD && kind <= REF_INVOKE_INTERFACE, "Invalid method handle kind {}!", kind);
        assert!(pool.has(reference_index as usize), "Invalid method handle reference index {}!", reference_index);
        if kind <= REF_PUT_STATIC { // FieldRef
            let reference = pool.get_field_ref(reference_index as usize)
                .expect(&format!("Invalid method handle! Expected field ref index {} to be in \
                    constant pool!", reference_index));
            return MethodHandle { kind, reference: MethodHandleRef::Field(reference) };
        }
        let reference = lookup_method_ref(pool, kind, reference_index, version);
        MethodHandle { kind, reference: MethodHandleRef::Method(reference) }
    }

    pub const fn new(kind: ReferenceKind, reference: MethodHandleRef) -> Self {
        MethodHandle { kind: kind as u8, reference }
    }

    pub fn is_field_ref(&self) -> bool {
        self.kind <= REF_PUT_STATIC
    }

    pub fn is_method_ref(&self) -> bool {
        self.kind >= REF_INVOKE_VIRTUAL
    }

    pub fn field_ref(&self) -> Option<Arc<FieldRef>> {
        self.reference.as_field().map(|value| Arc::clone(value))
    }

    pub fn method_ref(&self) -> Option<Arc<MethodRef>> {
        self.reference.as_method().map(|value| Arc::clone(value))
    }
}

fn lookup_method_ref(
    pool: &ConstantPool,
    kind: u8,
    index: u16,
    version: &ClassFileVersion
) -> Arc<MethodRef> {
    let reference = pool.get_method_ref(index as usize)
        .expect(&format!("Invalid method handle! Expected method ref index {} to be in constant \
            pool!", index));
    validate_method_ref(&reference, kind, version);
    reference
}

fn validate_method_ref(reference: &MethodRef, kind: u8, version: &ClassFileVersion) {
    if kind == REF_INVOKE_VIRTUAL || kind == REF_NEW_INVOKE_SPECIAL {
        assert!(!reference.is_interface, "Invalid method handle! Expected method reference to not \
            be an interface method reference!");
    }
    if (kind == REF_INVOKE_STATIC || kind == REF_INVOKE_SPECIAL) && version < &ClassFileVersion::RELEASE_8 {
        assert!(!reference.is_interface, "Invalid method handle! Expected method reference to not \
            be an interface method reference!");
    }
    if kind == REF_INVOKE_INTERFACE {
        assert!(reference.is_interface, "Invalid method handle! Expected method reference to be \
            an interface method reference!");
    }
    let name = reference.name();
    if (kind >= REF_INVOKE_VIRTUAL && kind <= REF_INVOKE_SPECIAL) || kind == REF_INVOKE_INTERFACE {
        assert_ne!(name, CLASS_INITIALIZER_METHOD_NAME, "Invalid method reference! invokeVirtual, \
            invokeStatic, invokeSpecial, and invokeInterface references cannot reference a \
            static initializer ({})!", CLASS_INITIALIZER_METHOD_NAME);
        assert_ne!(name, OBJECT_INITIALIZER_METHOD_NAME, "Invalid method reference! invokeVirtual, \
            invokeStatic, invokeSpecial, and invokeInterface references cannot reference a \
            constructor ({})!", OBJECT_INITIALIZER_METHOD_NAME);
    }
    if kind == REF_NEW_INVOKE_SPECIAL {
        assert_eq!(name, OBJECT_INITIALIZER_METHOD_NAME, "Invalid method reference! newInvokeSpecial \
            references must reference a constructor ({})!", OBJECT_INITIALIZER_METHOD_NAME);
    }
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum MethodHandleRef {
    Field(Arc<FieldRef>),
    Method(Arc<MethodRef>)
}

pub trait ElementRef: Debug {
    fn class(&self) -> &Class;

    fn name(&self) -> &str;
}

macro_rules! impl_element_ref {
    ($T:ident) => {
        impl ElementRef for $T {
            fn class(&self) -> &Class {
                &self.class
            }

            fn name(&self) -> &str {
                self.name.as_str()
            }
        }
    }
}

#[derive(Debug)]
pub struct FieldRef {
    class: Arc<Class>,
    name: IStr,
    descriptor: FieldDescriptor
}

impl FieldRef {
    pub const fn new(class: Arc<Class>, name: IStr, descriptor: FieldDescriptor) -> Self {
        FieldRef { class, name, descriptor }
    }

    pub fn descriptor(&self) -> &FieldDescriptor {
        &self.descriptor
    }
}

impl_element_ref!(FieldRef);

#[derive(Debug)]
pub struct MethodRef {
    class: Arc<Class>,
    name: IStr,
    descriptor: MethodDescriptor,
    is_interface: bool
}

impl MethodRef {
    pub const fn new(
        class: Arc<Class>,
        name: IStr,
        descriptor: MethodDescriptor,
        is_interface: bool
    ) -> Self {
        MethodRef { class, name, descriptor, is_interface }
    }

    pub fn descriptor(&self) -> &MethodDescriptor {
        &self.descriptor
    }

    pub fn is_interface(&self) -> bool {
        self.is_interface
    }
}

impl_element_ref!(MethodRef);

const REF_GET_FIELD: u8 = 1;
const REF_GET_STATIC: u8 = 2;
const REF_PUT_FIELD: u8 = 3;
const REF_PUT_STATIC: u8 = 4;
const REF_INVOKE_VIRTUAL: u8 = 5;
const REF_INVOKE_STATIC: u8 = 6;
const REF_INVOKE_SPECIAL: u8 = 7;
const REF_NEW_INVOKE_SPECIAL: u8 = 8;
const REF_INVOKE_INTERFACE: u8 = 9;

#[repr(u8)]
pub enum ReferenceKind {
    GetField = REF_GET_FIELD,
    GetStatic = REF_GET_STATIC,
    PutField = REF_PUT_FIELD,
    PutStatic = REF_PUT_STATIC,
    InvokeVirtual = REF_INVOKE_VIRTUAL,
    InvokeStatic = REF_INVOKE_STATIC,
    InvokeSpecial = REF_INVOKE_SPECIAL,
    NewInvokeSpecial = REF_NEW_INVOKE_SPECIAL,
    InvokeInterface = REF_INVOKE_INTERFACE
}
