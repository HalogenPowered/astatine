use std::sync::Arc;
use bytes::{Buf, Bytes};
use enum_as_inner::EnumAsInner;
use internship::IStr;
use java_desc::{Descriptor, FieldType, MethodType};
use paste::paste;
use crate::{Class, ClassLoader};
use crate::class_file::version::ClassFileVersion;
use crate::objects::handles::{FieldRef, MethodHandle, MethodRef};
use crate::types::method::BootstrapMethod;

macro_rules! get_constant {
    ($name:ident, $ty:ty) => {
        paste! {
            pub fn [<get_ $name>](&self, index: usize) -> Option<$ty> {
                self.get(index).and_then(|value| value.[<as_ $name>]()).map(|value| *value)
            }
        }
    }
}

macro_rules! get_ref {
    ($name:ident, $ty:ty, $as_name:ident) => {
        paste! {
            pub fn [<get_ $name>](&self, index: usize) -> Option<Arc<$ty>> {
                self.get(index).and_then(|value| value.[<as_ $name>]()).map(|value| Arc::clone(value))
            }
        }
    }
}

#[derive(Debug)]
pub struct ConstantPool {
    tags: Vec<u8>,
    constants: Vec<PoolConstant>
}

impl ConstantPool {
    pub(crate) fn parse(
        buf: &mut Bytes,
        loader: &ClassLoader,
        methods: &Vec<Arc<BootstrapMethod>>,
        version: &ClassFileVersion
    ) -> Self {
        let count = buf.get_u16();
        let mut tags = Vec::with_capacity(count as usize);
        let mut constants = Vec::with_capacity(count as usize);
        for _ in 0..count - 1 {
            let tag = buf.get_u8();
            tags.push(tag);
            let constant = PoolConstant::parse(tag, buf, methods, loader, &constants, version);
            constants.push(constant);
        }
        // No funny business on my watch!
        assert_eq!(tags.len(), constants.len(), "Tags and constants size mismatch!");
        ConstantPool::new(tags, constants)
    }

    pub const fn new(tags: Vec<u8>, constants: Vec<PoolConstant>) -> Self {
        ConstantPool { tags, constants }
    }

    pub fn len(&self) -> usize {
        self.tags.len()
    }

    pub fn has(&self, index: usize) -> bool {
        index < self.tags.len()
    }

    pub fn get_tag(&self, index: usize) -> Option<u8> {
        self.tags.get(index - 1).map(|value| *value)
    }

    fn get(&self, index: usize) -> Option<&PoolConstant> {
        self.constants.get(index - 1)
    }

    pub fn get_utf8(&self, index: usize) -> Option<&str> {
        self.get(index).and_then(|value| value.as_string()).map(|value| value.as_str())
    }

    // Same as get_utf8, but returns the underlying IStr object, rather than a splice
    pub fn get_string(&self, index: usize) -> Option<IStr> {
        self.get(index).and_then(|value| value.as_string()).map(|value| value.clone())
    }

    get_constant!(int, i32);
    get_constant!(float, f32);
    get_constant!(long, i64);
    get_constant!(double, f64);
    get_ref!(class, Class);
    get_ref!(field_ref, FieldRef);
    get_ref!(method_ref, MethodRef);
    get_ref!(interface_method_ref, MethodRef);
    get_ref!(method_handle, MethodHandle);
}

pub const UTF8_TAG: u8 = 1;
pub const INT_TAG: u8 = 3;
pub const FLOAT_TAG: u8 = 4;
pub const LONG_TAG: u8 = 5;
pub const DOUBLE_TAG: u8 = 6;
pub const CLASS_TAG: u8 = 7;
pub const STRING_TAG: u8 = 8;
pub const FIELD_REF_TAG: u8 = 9;
pub const METHOD_REF_TAG: u8 = 10;
pub const INTERFACE_METHOD_REF_TAG: u8 = 11;
pub const NAME_AND_TYPE_TAG: u8 = 12;
pub const METHOD_HANDLE_TAG: u8 = 15;
pub const METHOD_TYPE_TAG: u8 = 16;
pub const DYNAMIC_TAG: u8 = 17;
pub const INVOKE_DYNAMIC_TAG: u8 = 18;
pub const MODULE_TAG: u8 = 19;
pub const PACKAGE_TAG: u8 = 20;

#[derive(Debug, EnumAsInner)]
pub enum PoolConstant {
    Int(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class(Arc<Class>),
    String(IStr),
    FieldRef(Arc<FieldRef>),
    MethodRef(Arc<MethodRef>),
    InterfaceMethodRef(Arc<MethodRef>),
    MethodHandle(Arc<MethodHandle>),
    MethodType(MethodType),
    Dynamic(Arc<BootstrapMethod>, IStr, FieldType),
    InvokeDynamic(Arc<BootstrapMethod>, IStr, MethodType),
    Module(IStr),
    Package(IStr),
    NameAndType(IStr, Descriptor)
}

impl PoolConstant {
    fn parse(
        tag: u8,
        buf: &mut Bytes,
        methods: &Vec<Arc<BootstrapMethod>>,
        loader: &ClassLoader,
        pool: &Vec<PoolConstant>,
        version: &ClassFileVersion
    ) -> Self {
        match tag {
            UTF8_TAG => PoolConstant::String(PoolConstant::parse_utf8(buf)),
            INT_TAG => PoolConstant::Int(buf.get_i32()),
            FLOAT_TAG => PoolConstant::Float(buf.get_f32()),
            LONG_TAG => PoolConstant::Long(buf.get_i64()),
            DOUBLE_TAG => PoolConstant::Double(buf.get_f64()),
            CLASS_TAG => {
                let name_index = buf.get_u16();
                let name = pool.get(name_index as usize)
                    .and_then(|value| value.as_string())
                    .expect(&format!("Invalid name index {} for class tag!", name_index));
                PoolConstant::Class(loader.load_class(name.as_str()))
            },
            STRING_TAG => {
                let value_index = buf.get_u16();
                let value = pool.get(value_index as usize)
                    .and_then(|value| value.as_string())
                    .expect(&format!("Invalid value index {} for string tag!", value_index));
                PoolConstant::String(value.clone())
            },
            FIELD_REF_TAG => PoolConstant::parse_ref(buf, pool, |class, name, descriptor| {
                let field_type = field_type(descriptor, "name and type for field ref");
                PoolConstant::FieldRef(Arc::new(FieldRef::new(class, name, field_type)))
            }),
            METHOD_REF_TAG => {
                PoolConstant::MethodRef(PoolConstant::parse_method_ref(buf, pool, false))
            },
            INTERFACE_METHOD_REF_TAG => {
                PoolConstant::InterfaceMethodRef(PoolConstant::parse_method_ref(buf, pool, true))
            },
            NAME_AND_TYPE_TAG => {
                let name_index = buf.get_u16();
                let name = pool.get(name_index as usize)
                    .and_then(|value| value.as_string())
                    .expect(&format!("Invalid name index {} for name and type tag!", name_index));
                let descriptor_index = buf.get_u16();
                let descriptor = pool.get(descriptor_index as usize)
                    .and_then(|value| value.as_string())
                    .and_then(|value| Descriptor::parse(value.as_str()))
                    .expect(&format!("Invalid descriptor index {} for name and \
                        type tag!", descriptor_index));
                PoolConstant::NameAndType(name.clone(), descriptor)
            }
            METHOD_HANDLE_TAG => {
                PoolConstant::MethodHandle(Arc::new(MethodHandle::parse(pool, buf, version)))
            },
            METHOD_TYPE_TAG => {
                let descriptor_index = buf.get_u16();
                let descriptor = pool.get(descriptor_index as usize)
                    .and_then(|value| value.as_string())
                    .and_then(|value| MethodType::parse(value.as_str()))
                    .expect(&format!("Invalid method type tag! Expected descriptor string at \
                        index {} in constant pool!", descriptor_index));
                PoolConstant::MethodType(descriptor)
            },
            DYNAMIC_TAG => PoolConstant::parse_dynamic(pool, methods, buf, |method, name, descriptor| {
                let field_type = field_type(descriptor, "name and type for dynamic constant");
                PoolConstant::Dynamic(method, name, field_type)
            }),
            INVOKE_DYNAMIC_TAG => PoolConstant::parse_dynamic(pool, methods, buf, |method, name, descriptor| {
                let method_type = method_type(descriptor, "name and type for dynamic invocation");
                PoolConstant::InvokeDynamic(method, name, method_type)
            }),
            MODULE_TAG => {
                let name_index = buf.get_u16();
                let name = pool.get(name_index as usize)
                    .and_then(|value| value.as_string())
                    .expect(&format!("Invalid module tag! Expected index {} to be in constant \
                        pool!", name_index));
                PoolConstant::Module(name.clone())
            },
            PACKAGE_TAG => {
                let name_index = buf.get_u16();
                let name = pool.get(name_index as usize)
                    .and_then(|value| value.as_string())
                    .expect(&format!("Invalid module tag! Expected index {} to be in constant \
                        pool!", name_index));
                PoolConstant::Package(name.clone())
            }
            _ => panic!("Invalid tag {} for constant pool entry!", tag)
        }
    }

    fn parse_utf8(buf: &mut Bytes) -> IStr {
        let length = buf.get_u16();
        let bytes = buf.copy_to_bytes(length as usize).to_vec();
        IStr::from_utf8(bytes.as_slice()).expect("Failed to convert bytes to string!")
    }

    fn parse_method_ref(buf: &mut Bytes, values: &Vec<PoolConstant>, is_interface: bool) -> Arc<MethodRef> {
        PoolConstant::parse_ref(buf, values, |class, name, descriptor| {
            let method_type = method_type(descriptor, "name and type for method/interface method ref");
            Arc::new(MethodRef::new(class, name, method_type, is_interface))
        })
    }

    fn parse_ref<T, F>(
        buf: &mut Bytes,
        values: &Vec<PoolConstant>,
        mapper: F
    ) -> T where F: FnOnce(Arc<Class>, IStr, &Descriptor) -> T {
        let class_index = buf.get_u16();
        let class = values.get(class_index as usize)
            .and_then(|value| value.as_class())
            .expect(&format!("Invalid class index {} for field ref tag!", class_index));
        let name_and_type_index = buf.get_u16();
        let name_and_type = values.get(name_and_type_index as usize)
            .and_then(|value| value.as_name_and_type())
            .expect(&format!("Invalid name and type index {} for field ref \
                        tag!", name_and_type_index));
        mapper(Arc::clone(class), name_and_type.0.clone(), name_and_type.1)
    }

    fn parse_dynamic<T, F>(
        pool: &Vec<PoolConstant>,
        bootstrap_methods: &Vec<Arc<BootstrapMethod>>,
        buf: &mut Bytes,
        mapper: F
    ) -> T where F: FnOnce(Arc<BootstrapMethod>, IStr, &Descriptor) -> T {
        let bootstrap_method_index = buf.get_u16();
        let bootstrap_method = bootstrap_methods.get(bootstrap_method_index as usize)
            .expect(&format!("Invalid dynamic constant! Expected index {} to be in constant \
                pool!", bootstrap_method_index));
        let name_and_type_index = buf.get_u16();
        let name_and_type = pool.get(name_and_type_index as usize)
            .and_then(|value| value.as_name_and_type())
            .expect(&format!("Invalid dynamic constant! Expected index {} to be in constant \
                pool!", name_and_type_index));
        mapper(Arc::clone(bootstrap_method), name_and_type.0.clone(), name_and_type.1)
    }
}

macro_rules! descriptor_as {
    ($name:ident, $T:ident, $D:ident, $type_name:literal) => {
        fn $name(descriptor: &Descriptor, error: &str) -> $T {
            match descriptor {
                Descriptor::$D(element_type) => element_type.clone(),
                _ => panic!("Invalid {} tag! Descriptor must be a {} descriptor!", error, $type_name)
            }
        }
    }
}

descriptor_as!(field_type, FieldType, Field, "field");
descriptor_as!(method_type, MethodType, Method, "method");

fn is_loadable(tag: u8) -> bool {
    tag == INT_TAG || tag == FLOAT_TAG || tag == LONG_TAG || tag == DOUBLE_TAG ||
        tag == CLASS_TAG || tag == STRING_TAG || tag == METHOD_HANDLE_TAG ||
        tag == METHOD_TYPE_TAG || tag == DYNAMIC_TAG
}
