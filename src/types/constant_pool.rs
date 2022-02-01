use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use bytes::{Buf, Bytes};
use enum_as_inner::EnumAsInner;
use internship::IStr;
use java_desc::{FieldType, MethodType};
use paste::paste;
use crate::{Class, ClassLoader};
use crate::objects::handles::{FieldRef, MethodHandle, MethodRef};
use crate::types::method::BootstrapMethod;
use crate::utils::lateinit::LateInit;

macro_rules! get_constant {
    ($name:ident, $ty:ty) => {
        paste! {
            pub fn [<get_ $name>](&self, index: usize) -> Option<$ty> {
                self.get(index).and_then(|value| value.[<as_ $name>]()).map(|value| *value)
            }
        }
    }
}

macro_rules! get_index {
    ($name:ident) => {
        paste! {
            fn [<get_ $name _index>](&self, index: usize) -> Option<u16> {
                self.get(index).and_then(|value| value.[<as_ $name>]()).map(|value| *value)
            }
        }
    }
}

macro_rules! get_tuple_index {
    ($name:ident, $as_name:ident) => {
        paste! {
            fn [<get_ $name _indices>](&self, index: usize) -> Option<(u16, u16)> {
                self.get(index)
                    .and_then(|value| value.$as_name())
                    .map(|value| (*value.0, *value.1))
            }
        }
    };
    ($name:ident) => {
        paste! { get_tuple_index!($name, [<as_ $name>]); }
    }
}

#[derive(Debug)]
pub struct ConstantPool {
    holder: LateInit<Arc<Class>>,
    tags: Vec<u8>,
    constants: Vec<PoolConstant>,
    resolution_cache: RwLock<HashMap<usize, Option<ResolvedPoolConstant>>>,
    has_dynamic: bool
}

impl ConstantPool {
    pub(crate) fn parse(buf: &mut Bytes) -> Self {
        let count = buf.get_u16();
        let mut tags = Vec::with_capacity(count as usize);
        let mut constants = Vec::with_capacity(count as usize);

        let mut index = 1;
        let mut has_dynamic = false;
        while index < count {
            let tag = buf.get_u8();
            if tag == DYNAMIC_TAG || tag == INVOKE_DYNAMIC_TAG { has_dynamic = true }
            tags.push(tag);
            constants.push(PoolConstant::parse(tag, buf));
            if tag == LONG_TAG || tag == DOUBLE_TAG { index += 2 } else { index += 1 }
        }

        // No funny business on my watch!
        assert_eq!(tags.len(), constants.len(), "Tags and constants size mismatch!");
        ConstantPool::new(tags, constants, has_dynamic)
    }

    fn new(tags: Vec<u8>, constants: Vec<PoolConstant>, has_dynamic: bool) -> Self {
        ConstantPool {
            holder: LateInit::new(),
            tags,
            constants,
            resolution_cache: RwLock::new(HashMap::new()),
            has_dynamic
        }
    }

    pub fn holder(&self) -> Arc<Class> {
        Arc::clone(&self.holder)
    }

    pub(crate) fn set_holder(&self, class: Arc<Class>) {
        self.holder.init(class)
    }

    pub fn has_dynamic(&self) -> bool {
        self.has_dynamic
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

    pub fn get_utf8(&self, index: usize) -> Option<IStr> {
        self.get(index).and_then(|value| value.as_utf8()).map(|value| value.clone())
    }

    pub fn get_string(&self, index: usize) -> Option<IStr> {
        let resolver = || {
            let index = self.get_string_index(index)?;
            let string = self.get_utf8(index as usize)?;
            Some(ResolvedPoolConstant::String(string))
        };
        let converter = |value: &ResolvedPoolConstant| value.as_string().map(|value| value.clone());
        self.resolve(index, resolver, converter)
    }

    pub fn get_class(&self, index: usize) -> Option<Arc<Class>> {
        let resolver = || {
            let index = self.get_class_index(index)?;
            let class_name = self.get_utf8(index as usize)?;
            let class = self.holder.loader().load_class(class_name.as_str());
            Some(ResolvedPoolConstant::Class(class))
        };
        let converter = |value: &ResolvedPoolConstant| value.as_class()
            .map(|value| Arc::clone(value));
        self.resolve(index, resolver, converter)
    }

    pub fn get_field_ref(&self, index: usize) -> Option<Arc<FieldRef>> {
        let resolver = || {
            let (class_index, nat_index) = self.get_field_ref_indices(index)?;
            Some(ResolvedPoolConstant::FieldRef(parse_field_ref(self, class_index, nat_index)))
        };
        let converter = |value: &ResolvedPoolConstant| value.as_field_ref()
            .map(|value| Arc::clone(value));
        self.resolve(index, resolver, converter)
    }

    pub fn get_method_ref(&self, index: usize) -> Option<Arc<MethodRef>> {
        let resolver = || {
            let (class_index, nat_index, is_interface) = self.get_unresolved_method_ref(index)?;
            let method_ref = parse_method_ref(self, class_index, nat_index, is_interface);
            Some(ResolvedPoolConstant::MethodRef(method_ref))
        };
        let converter = |value: &ResolvedPoolConstant| value.as_method_ref()
            .map(|value| Arc::clone(value));
        self.resolve(index, resolver, converter)
    }

    pub fn get_method_handle(&self, index: usize) -> Option<Arc<MethodHandle>> {
        let resolver = || {
            let (kind, ref_index) = self.get_unresolved_method_handle(index)?;
            let handle = MethodHandle::parse(self, kind, ref_index, self.holder.version());
            Some(ResolvedPoolConstant::MethodHandle(Arc::new(handle)))
        };
        let converter = |value: &ResolvedPoolConstant| value.as_method_handle()
            .map(|value| Arc::clone(value));
        self.resolve(index, resolver, converter)
    }

    pub fn get_method_type(&self, index: usize) -> Option<MethodType> {
        let resolver = || {
            let descriptor_index = self.get_method_type_index(index)?;
            let descriptor = self.get_utf8(descriptor_index as usize)
                .and_then(|value| MethodType::parse(value.as_str()))?;
            Some(ResolvedPoolConstant::MethodType(descriptor))
        };
        let converter = |value: &ResolvedPoolConstant| value.as_method_type()
            .map(|value| value.clone());
        self.resolve(index, resolver, converter)
    }

    get_index!(class);
    get_index!(string);
    get_index!(method_type);
    get_tuple_index!(nat, as_name_and_type);
    get_tuple_index!(field_ref);

    fn get_unresolved_method_ref(&self, index: usize) -> Option<(u16, u16, bool)> {
        match self.get(index) {
            Some(PoolConstant::MethodRef { class_index, nat_index }) =>
                Some((*class_index, *nat_index, false)),
            Some(PoolConstant::InterfaceMethodRef { class_index, nat_index }) =>
                Some((*class_index, *nat_index, true)),
            _ => None
        }
    }

    fn get_unresolved_method_handle(&self, index: usize) -> Option<(u8, u16)> {
        self.get(index).and_then(|value| value.as_method_handle()).map(|value| (*value.0, *value.1))
    }

    get_constant!(int, i32);
    get_constant!(float, f32);
    get_constant!(long, i64);
    get_constant!(double, f64);

    fn get(&self, index: usize) -> Option<&PoolConstant> {
        self.constants.get(index - 1)
    }

    pub(crate) fn get_class_name(&self, index: usize) -> Option<IStr> {
        self.get_class_index(index).and_then(|value| self.get_utf8(value as usize))
    }

    pub(crate) fn get_class_no_holder(&self, index: usize, loader: Arc<ClassLoader>) -> Option<Arc<Class>> {
        let resolver = || {
            let index = self.get_class_index(index)?;
            let class_name = self.get_utf8(index as usize)?;
            let class = loader.load_class(class_name.as_str());
            Some(ResolvedPoolConstant::Class(class))
        };
        let converter = |value: &ResolvedPoolConstant| value.as_class()
            .map(|value| Arc::clone(value));
        self.resolve(index, resolver, converter)
    }

    fn resolve<T: Clone>(
        &self,
        index: usize,
        resolver: impl FnOnce() -> Option<ResolvedPoolConstant>,
        converter: impl FnOnce(&ResolvedPoolConstant) -> Option<T>
    ) -> Option<T> {
        self.resolution_cache.write().unwrap()
            .entry(index)
            .or_insert_with(resolver)
            .as_ref()
            .and_then(converter)
    }
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
enum PoolConstant {
    Utf8(IStr),
    Int(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class { name_index: u16 },
    String { value_index: u16 },
    FieldRef { class_index: u16, nat_index: u16 },
    MethodRef { class_index: u16, nat_index: u16 },
    InterfaceMethodRef { class_index: u16, nat_index: u16 },
    NameAndType { name_index: u16, descriptor_index: u16 },
    MethodHandle { reference_kind: u8, reference_index: u16 },
    MethodType { descriptor_index: u16 },
    Dynamic { bootstrap_method_index: u16, nat_index: u16 },
    InvokeDynamic { bootstrap_method_index: u16, nat_index: u16 },
    Module { name_index: u16 },
    Package { name_index: u16 }
}

impl PoolConstant {
    fn parse(tag: u8, buf: &mut Bytes) -> Self {
        match tag {
            UTF8_TAG => PoolConstant::Utf8(PoolConstant::parse_utf8(buf)),
            INT_TAG => PoolConstant::Int(buf.get_i32()),
            FLOAT_TAG => PoolConstant::Float(buf.get_f32()),
            LONG_TAG => PoolConstant::Long(buf.get_i64()),
            DOUBLE_TAG => PoolConstant::Double(buf.get_f64()),
            CLASS_TAG => PoolConstant::Class { name_index: buf.get_u16() },
            STRING_TAG => PoolConstant::String { value_index: buf.get_u16() },
            FIELD_REF_TAG => PoolConstant::FieldRef {
                class_index: buf.get_u16(),
                nat_index: buf.get_u16()
            },
            METHOD_REF_TAG => PoolConstant::MethodRef {
                class_index: buf.get_u16(),
                nat_index: buf.get_u16()
            },
            INTERFACE_METHOD_REF_TAG => PoolConstant::InterfaceMethodRef {
                class_index: buf.get_u16(),
                nat_index: buf.get_u16()
            },
            NAME_AND_TYPE_TAG => PoolConstant::NameAndType {
                name_index: buf.get_u16(),
                descriptor_index: buf.get_u16()
            },
            METHOD_HANDLE_TAG => PoolConstant::MethodHandle {
                reference_kind: buf.get_u8(),
                reference_index: buf.get_u16()
            },
            METHOD_TYPE_TAG => PoolConstant::MethodType { descriptor_index: buf.get_u16() },
            DYNAMIC_TAG => PoolConstant::Dynamic {
                bootstrap_method_index: buf.get_u16(),
                nat_index: buf.get_u16()
            },
            INVOKE_DYNAMIC_TAG => PoolConstant::InvokeDynamic {
                bootstrap_method_index: buf.get_u16(),
                nat_index: buf.get_u16()
            },
            MODULE_TAG => PoolConstant::Module { name_index: buf.get_u16() },
            PACKAGE_TAG => PoolConstant::Package { name_index: buf.get_u16() },
            _ => panic!("Invalid tag {} for constant pool entry!", tag)
        }
    }

    fn parse_utf8(buf: &mut Bytes) -> IStr {
        let length = buf.get_u16();
        let bytes = buf.copy_to_bytes(length as usize).to_vec();
        IStr::from_utf8(bytes.as_slice()).expect("Failed to convert bytes to string!")
    }
}

#[derive(Debug, EnumAsInner)]
enum ResolvedPoolConstant {
    Class(Arc<Class>),
    String(IStr),
    FieldRef(Arc<FieldRef>),
    MethodRef(Arc<MethodRef>),
    MethodHandle(Arc<MethodHandle>),
    MethodType(MethodType),
    Dynamic(Arc<BootstrapMethod>, IStr, FieldType),
    InvokeDynamic(Arc<BootstrapMethod>, IStr, MethodType)
}

fn parse_field_ref(pool: &ConstantPool, class_index: u16, nat_index: u16) -> Arc<FieldRef> {
    let mapper = |string: IStr| FieldType::parse(string.as_str());
    parse_ref(pool, class_index, nat_index, mapper, FieldRef::new)
}

fn parse_method_ref(
    pool: &ConstantPool,
    class_index: u16,
    nat_index: u16,
    is_interface: bool
) -> Arc<MethodRef> {
    let mapper = |string: IStr| MethodType::parse(string.as_str());
    parse_ref(pool, class_index, nat_index, mapper, |class, name, descriptor| {
        MethodRef::new(class, name, descriptor, is_interface)
    })
}

fn parse_ref<T, D>(
    pool: &ConstantPool,
    class_index: u16,
    nat_index: u16,
    mapper: impl FnOnce(IStr) -> Option<D>,
    constructor: impl FnOnce(Arc<Class>, IStr, D) -> T
) -> Arc<T> {
    let class = pool.get_class(class_index as usize)
        .expect(&format!("Invalid class index {} for ref tag!", class_index));
    let (name_index, descriptor_index) = pool.get_nat_indices(nat_index as usize)
        .expect(&format!("Invalid name and type index {} for ref tag!", nat_index));
    let name = pool.get_string(name_index as usize)
        .expect(&format!("Invalid name index {} for ref tag!", name_index));
    let descriptor = pool.get_string(descriptor_index as usize)
        .and_then(mapper)
        .expect(&format!("Invalid descriptor index {} for ref tag!", descriptor_index));
    Arc::new(constructor(class, name, descriptor))
}
