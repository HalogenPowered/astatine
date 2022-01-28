use std::fmt::Debug;
use std::sync::Arc;
use bytes::{Buf, Bytes};
use enum_as_inner::EnumAsInner;
use internship::IStr;
use java_desc::{FieldType, MethodType};
use crate::Class;
use crate::class_file::version::ClassFileVersion;
use crate::types::constant_pool::PoolConstant;
use crate::utils::constants::{CLASS_INITIALIZER_METHOD_NAME, OBJECT_INITIALIZER_METHOD_NAME};

#[derive(Debug)]
pub struct MethodHandle {
    kind: ReferenceKind,
    reference: MethodHandleRef,
}

impl MethodHandle {
    pub(crate) fn parse(
        pool: &Vec<PoolConstant>,
        buf: &mut Bytes,
        version: &ClassFileVersion
    ) -> Self {
        let kind = buf.get_u8();
        let reference_kind = ReferenceKind::from(kind);
        let reference_index = buf.get_u16();
        if kind <= REF_PUT_STATIC { // FieldRef
            let reference = pool.get(reference_index as usize)
                .and_then(|value| value.as_field_ref())
                .expect(&format!("Invalid method handle! Expected field ref index {} to be in \
                    constant pool!", reference_index));
            return MethodHandle::new(reference_kind, MethodHandleRef::Field(Arc::clone(reference)));
        }
        let interface_ref_status = InterfaceRefStatus::from_kind(kind, version);
        let reference = lookup_method_ref(pool, reference_index, interface_ref_status);
        validate_method_ref(kind, reference);
        MethodHandle::new(reference_kind, MethodHandleRef::Method(Arc::clone(reference)))
    }

    pub const fn new(kind: ReferenceKind, reference: MethodHandleRef) -> Self {
        MethodHandle { kind, reference }
    }

    pub fn is_field_ref(&self) -> bool {
        self.kind.is_field_ref()
    }

    pub fn is_method_ref(&self) -> bool {
        self.kind.is_method_ref()
    }

    pub fn field_ref(&self) -> Option<Arc<FieldRef>> {
        self.reference.as_field().map(|value| Arc::clone(value))
    }

    pub fn method_ref(&self) -> Option<Arc<MethodRef>> {
        self.reference.as_method().map(|value| Arc::clone(value))
    }
}

fn lookup_method_ref<'a>(
    pool: &'a Vec<PoolConstant>,
    index: u16,
    status: InterfaceRefStatus
) -> &'a Arc<MethodRef> {
    let transformer = |value: &'a PoolConstant| {
        match status {
            InterfaceRefStatus::Required => value.as_interface_method_ref(),
            InterfaceRefStatus::Allowed => value.as_interface_method_ref()
                .or(value.as_method_ref()),
            InterfaceRefStatus::Denied => value.as_method_ref()
        }
    };
    pool.get(index as usize)
        .and_then(transformer)
        .expect(&format!("Invalid method handle! Expected method ref index {} to be in constant \
            pool!", index))
}

fn validate_method_ref(kind: u8, reference: &Arc<MethodRef>) {
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

enum InterfaceRefStatus {
    Required,
    Allowed,
    Denied
}

impl InterfaceRefStatus {
    fn from_kind(kind: u8, version: &ClassFileVersion) -> InterfaceRefStatus {
        if kind == REF_INVOKE_VIRTUAL || kind == REF_NEW_INVOKE_SPECIAL {
            return InterfaceRefStatus::Denied;
        }
        if kind == REF_INVOKE_STATIC || kind == REF_INVOKE_SPECIAL {
            return if version < &ClassFileVersion::RELEASE_8 {
                InterfaceRefStatus::Denied
            } else {
                InterfaceRefStatus::Allowed
            }
        }
        if kind == REF_INVOKE_INTERFACE {
            return InterfaceRefStatus::Required;
        }
        panic!("Invalid kind {}!", kind)
    }
}

#[derive(Debug, EnumAsInner)]
pub enum MethodHandleRef {
    Field(Arc<FieldRef>),
    Method(Arc<MethodRef>)
}

pub trait ElementRef: Debug {
    fn class(&self) -> Arc<Class>;

    fn name(&self) -> &str;
}

pub trait FieldElementRef: ElementRef {
    fn descriptor(&self) -> &FieldType;
}

pub trait MethodElementRef: ElementRef {
    fn descriptor(&self) -> &MethodType;
}

macro_rules! impl_element_ref {
    ($T:ident) => {
        impl ElementRef for $T {
            fn class(&self) -> Arc<Class> {
                Arc::clone(&self.class)
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
    descriptor: FieldType
}

impl FieldRef {
    pub const fn new(class: Arc<Class>, name: IStr, descriptor: FieldType) -> Self {
        FieldRef { class, name, descriptor }
    }
}

impl FieldElementRef for FieldRef {
    fn descriptor(&self) -> &FieldType {
        &self.descriptor
    }
}

impl_element_ref!(FieldRef);

#[derive(Debug)]
pub struct MethodRef {
    class: Arc<Class>,
    name: IStr,
    descriptor: MethodType,
    is_interface: bool
}

impl MethodRef {
    pub const fn new(
        class: Arc<Class>,
        name: IStr,
        descriptor: MethodType,
        is_interface: bool
    ) -> Self {
        MethodRef { class, name, descriptor, is_interface }
    }

    pub fn is_interface(&self) -> bool {
        self.is_interface
    }
}

impl MethodElementRef for MethodRef {
    fn descriptor(&self) -> &MethodType {
        &self.descriptor
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

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, EnumAsInner)]
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

impl ReferenceKind {
    pub fn from(value: u8) -> ReferenceKind {
        match value {
            REF_GET_FIELD => ReferenceKind::GetField,
            REF_GET_STATIC => ReferenceKind::GetStatic,
            REF_PUT_FIELD => ReferenceKind::PutField,
            REF_PUT_STATIC => ReferenceKind::PutStatic,
            REF_INVOKE_VIRTUAL => ReferenceKind::InvokeVirtual,
            REF_INVOKE_STATIC => ReferenceKind::InvokeStatic,
            REF_INVOKE_SPECIAL => ReferenceKind::InvokeSpecial,
            REF_NEW_INVOKE_SPECIAL => ReferenceKind::NewInvokeSpecial,
            REF_INVOKE_INTERFACE => ReferenceKind::InvokeInterface,
            _ => panic!("Invalid reference kind {}!", value)
        }
    }

    pub fn is_field_ref(&self) -> bool {
        *self as u8 <= REF_PUT_STATIC
    }

    pub fn is_method_ref(&self) -> bool {
        !self.is_field_ref()
    }
}
