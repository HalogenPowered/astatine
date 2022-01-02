use bytes::{Buf, Bytes};

pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<ElementInfo>,
    pub methods: Vec<ElementInfo>,
    pub attributes: Vec<AttributeInfo>
}

pub type ConstantPool = Vec<ConstantPoolEntry>;

pub struct ConstantPoolEntry {
    pub tag: u8,
    pub info: dyn ConstantPoolInfo
}

pub const UTF8_TAG: u8 = 1;
pub const INTEGER_TAG: u8 = 3;
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

// I hope there's a better way to do this, but this allows us to pass in implementations here
// that we can then pattern match later
pub trait ConstantPoolInfo {
    fn read_from(tag: u8, mut buf: &Bytes) -> Self;
}

// For Class, Module, and Package
pub struct StructureConstantInfo {
    pub tag: u8,
    pub name_index: u16
}

impl ConstantPoolInfo for StructureConstantInfo {
    fn read_from(tag: u8, mut buf: &Bytes) -> Self {
        let name_index = buf.get_u16();
        StructureConstantInfo { tag, name_index }
    }
}

// FieldRef, MethodRef, and InterfaceMethodRef are all the same, so we use this to represent those
pub struct ElementRefConstantInfo {
    pub tag: u8,
    pub class_index: u16,
    pub name_and_type_index: u16
}

impl ConstantPoolInfo for ElementRefConstantInfo {
    fn read_from(tag: u8, mut buf: &Bytes) -> Self {
        let class_index = buf.get_u16();
        let name_and_type_index = buf.get_u16();
        ElementRefConstantInfo { tag, class_index, name_and_type_index }
    }
}

pub struct StringConstantInfo {
    pub tag: u8,
    pub string_index: u16
}

impl ConstantPoolInfo for StringConstantInfo {
    fn read_from(tag: u8, mut buf: &Bytes) -> Self {
        let string_index = buf.get_u16();
        StringConstantInfo { tag, string_index }
    }
}

// For Integer and Float, Single means a single 32-bit number.
// error 404: better_name not found
pub struct SinglePrimitiveConstantInfo {
    pub tag: u8,
    pub bytes: u32
}

impl ConstantPoolInfo for SinglePrimitiveConstantInfo {
    fn read_from(tag: u8, mut buf: &Bytes) -> Self {
        let bytes = buf.get_u32();
        SinglePrimitiveConstantInfo { tag, bytes }
    }
}

// For Long and Double, Double means two 32-bit numbers.
// error 404: better_name not found
pub struct DoublePrimitiveConstantInfo {
    pub tag: u8,
    pub high_bytes: u32,
    pub low_bytes: u32
}

impl ConstantPoolInfo for DoublePrimitiveConstantInfo {
    fn read_from(tag: u8, mut buf: &Bytes) -> Self {
        let high_bytes = buf.get_u32();
        let low_bytes = buf.get_u32();
        DoublePrimitiveConstantInfo { tag, high_bytes, low_bytes }
    }
}

pub struct NameAndTypeConstantInfo {
    pub tag: u8,
    pub name_index: u16,
    pub descriptor_index: u16
}

impl ConstantPoolInfo for NameAndTypeConstantInfo {
    fn read_from(tag: u8, mut buf: &Bytes) -> Self {
        let name_index = buf.get_u16();
        let descriptor_index = buf.get_u16();
        NameAndTypeConstantInfo { tag, name_index, descriptor_index }
    }
}

pub struct Utf8ConstantInfo {
    pub tag: u8,
    pub value: &'static str
}

impl ConstantPoolInfo for Utf8ConstantInfo {
    fn read_from(tag: u8, mut buf: &Bytes) -> Self {
        let length = buf.get_u16();
        let bytes = buf.copy_to_bytes(length as usize).to_vec();
        let value = String::from_utf8(bytes).expect("Failed to convert bytes to string!");
        Utf8ConstantInfo { tag, value: &value }
    }
}

pub struct MethodHandleConstantInfo {
    pub tag: u8,
    pub reference_kind: u8,
    pub reference_index: u16
}

impl ConstantPoolInfo for MethodHandleConstantInfo {
    fn read_from(tag: u8, mut buf: &Bytes) -> Self {
        let reference_kind = buf.get_u8();
        let reference_index = buf.get_u16();
        MethodHandleConstantInfo { tag, reference_kind, reference_index }
    }
}

pub struct MethodTypeConstantInfo {
    pub tag: u8,
    pub descriptor_index: u16
}

impl ConstantPoolInfo for MethodTypeConstantInfo {
    fn read_from(tag: u8, mut buf: &Bytes) -> Self {
        let descriptor_index = buf.get_u16();
        MethodTypeConstantInfo { tag, descriptor_index }
    }
}

// For both Dynamic and InvokeDynamic
pub struct DynamicConstantInfo {
    pub tag: u8,
    pub bootstrap_method_attr_index: u16,
    pub name_and_type_index: u16
}

impl ConstantPoolInfo for DynamicConstantInfo {
    fn read_from(tag: u8, mut buf: &Bytes) -> Self {
        let bootstrap_method_attr_index = buf.get_u16();
        let name_and_type_index = buf.get_u16();
        DynamicConstantInfo { tag, bootstrap_method_attr_index, name_and_type_index }
    }
}

// Used for both field_info and method_info.
// They have the same fields in the same order with the same types, there's no point repeating ourselves.
pub struct ElementInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>
}

impl ElementInfo {
    fn read_from(mut buf: &Bytes) -> Self {
        let access_flags = buf.get_u16();
        let name_index = buf.get_u16();
        let descriptor_index = buf.get_u16();
        let attributes_count = buf.get_u16();
    }
}

pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub info: Vec<u8>
}
