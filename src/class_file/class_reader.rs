use bytes::{Bytes, Buf};
use std::fs;
use crate::class_file::attribute_tags::{TAG_INNER_CLASSES, TAG_RECORD, TAG_SOURCE_FILE};
use crate::class_file::fields::parse_field;
use crate::class_file::methods::parse_method;
use crate::types::access_flags::*;
use crate::types::class::{Class, InnerClassInfo};
use crate::types::constant_pool::ConstantPool;
use crate::types::record::RecordComponent;
use crate::utils::constants::{JAVA_VERSION_1_5, JAVA_VERSION_6, JAVA_VERSION_9};

const MAGIC_CLASS_FILE_VERSION: u32 = 0xCAFEBABE;

pub fn parse_class(class_file_name: &str) -> Class {
    let contents = fs::read(class_file_name)
        .expect(&format!("Class file name {} could not be read!", class_file_name));
    let mut buf = Bytes::from(contents);
    let magic = buf.get_u32();
    if magic != MAGIC_CLASS_FILE_VERSION {
        panic!("Invalid class file {}! Expected magic header {}, got {}!", class_file_name, MAGIC_CLASS_FILE_VERSION, magic);
    }
    let minor_version = buf.get_u16();
    let major_version = buf.get_u16();
    let constant_pool = ConstantPool::parse(&mut buf);

    let mut access_flags = if major_version >= JAVA_VERSION_9 {
        buf.get_u16() & ALL_CLASS_MODIFIERS_J9
    } else {
        buf.get_u16() & ALL_CLASS_MODIFIERS
    };
    if (access_flags & ACC_INTERFACE) != 0 && major_version < JAVA_VERSION_6 {
        // Set abstract flag for backwards compatibility
        access_flags |= ACC_ABSTRACT;
    }
    verify_modifiers(class_file_name, major_version, access_flags);

    let this_class = buf.get_u16();
    let name = constant_pool.resolve_class_name(this_class as usize)
        .expect(&format!("Invalid class file {}! Expected class constant to be at index {} in \
            constant pool!", class_file_name, this_class))
        .clone();
    let super_class = buf.get_u16();

    // Parse super interface indices
    let interfaces_count = buf.get_u16();
    let mut interfaces = Vec::with_capacity(interfaces_count as usize);
    for _ in 0..interfaces_count {
        interfaces.push(buf.get_u16());
    }

    // Parse fields
    let fields_count = buf.get_u16();
    let mut fields = Vec::with_capacity(fields_count as usize);
    for _ in 0..fields_count {
        fields.push(parse_field(class_file_name, major_version, &constant_pool, &mut buf));
    }

    // Parse methods
    let is_interface = (access_flags & ACC_INTERFACE) != 0;
    let methods_count = buf.get_u16();
    let mut methods = Vec::with_capacity(methods_count as usize);
    for _ in 0..methods_count {
        methods.push(parse_method(class_file_name, major_version, is_interface, &constant_pool, &mut buf));
    }

    let attributes_count = buf.get_u16();
    let (source_file_name, inner_classes, record_components) = parse_attributes(
        class_file_name,
        &constant_pool,
        attributes_count,
        &mut buf
    );

    Class {
        major_version,
        minor_version,
        access_flags,
        constant_pool,
        name,
        super_class,
        interfaces,
        fields,
        methods,
        source_file_name,
        inner_classes,
        record_components
    }
}

fn parse_attributes(
    class_file_name: &str,
    pool: &ConstantPool,
    mut attribute_count: u16,
    buf: &mut Bytes
) -> (Option<String>, Option<Vec<InnerClassInfo>>, Option<Vec<RecordComponent>>) {
    let mut source_file_name = None;
    let mut inner_classes = None;
    let mut record_components = None;

    while attribute_count > 0 {
        assert!(buf.len() >= 6, "Truncated class attributes for class file {}!", class_file_name);
        let attribute_name_index = buf.get_u16();
        let attribute_length = buf.get_u32();
        let attribute_name = pool.get_utf8(attribute_name_index as usize)
            .expect(&format!("Invalid class attribute index {} for class file {}! Expected name \
                to be in constant pool!", attribute_name_index, class_file_name));

        if attribute_name == TAG_SOURCE_FILE {
            assert_eq!(attribute_length, 2, "Invalid source file attribute for class file {}! Expected \
                length of 2, was {}!", class_file_name, attribute_length);
            assert!(source_file_name.is_none(), "Duplicate source file attribute found for class file {}!", class_file_name);
            let source_file_index = buf.get_u16();
            let source_file = pool.get_string(source_file_index as usize)
                .expect(&format!("Invalid source file attribute for class file {}! Expected name \
                    index {} to be in constant pool!", class_file_name, source_file_index))
                .clone();
            source_file_name = Some(source_file);
        } else if attribute_name == TAG_INNER_CLASSES {
            assert!(inner_classes.is_none(), "Duplicate inner classes attribute found for class file {}!", class_file_name);
            let number_of_classes = buf.get_u16();
            let mut classes = Vec::with_capacity(number_of_classes as usize);
            for _ in 0..number_of_classes {
                classes.push(InnerClassInfo::parse(class_file_name, pool, buf));
            }
            inner_classes = Some(classes);
        } else if attribute_name == TAG_RECORD {
            assert!(record_components.is_none(), "Duplicate record attribute found for class file {}!", class_file_name);
            let components_count = buf.get_u16();
            let mut components = Vec::with_capacity(components_count as usize);
            for _ in 0..components_count {
                components.push(RecordComponent::parse(class_file_name, pool, buf));
            }
            record_components = Some(components);
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attribute_count -= 1;
    }
    (source_file_name, inner_classes, record_components)
}

fn verify_modifiers(class_file_name: &str, major_version: u16, flags: u16) {
    let is_module = (flags & ACC_MODULE) != 0;
    assert!(major_version >= JAVA_VERSION_9 || !is_module, "Invalid class file {}! Module flag should \
        not be set for classes before Java 9!", class_file_name);
    assert!(!is_module, "Cannot load class file {} as it is a module!", class_file_name);

    let is_final = (flags & ACC_FINAL) != 0;
    let is_super = (flags & ACC_SUPER) != 0;
    let is_interface = (flags & ACC_INTERFACE) != 0;
    let is_abstract = (flags & ACC_ABSTRACT) != 0;
    let is_annotation = (flags & ACC_ANNOTATION) != 0;
    let is_enum = (flags & ACC_ENUM) != 0;
    let major_1_5_or_above = major_version >= JAVA_VERSION_1_5;

    let is_illegal = (is_abstract && is_final) ||
        (is_interface && !is_abstract) ||
        (is_interface && major_1_5_or_above && (is_super || is_enum)) ||
        (!is_interface && major_1_5_or_above && is_annotation);
    assert!(!is_illegal, "Invalid class file {}! Illegal class modifiers {}!", class_file_name, flags);
}
