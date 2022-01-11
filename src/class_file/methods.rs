use std::collections::HashMap;
use bytes::{Buf, Bytes};
use java_desc::MethodType;

use crate::class_file::attribute_tags::*;
use crate::class_file::utils::parse_generic_signature;
use crate::types::access_flags::*;
use crate::class_file::code::*;
use crate::class_file::stack_map_table::{parse_stack_map_table, StackMapTable};
use crate::types::constant_pool::ConstantPool;
use crate::types::method::*;
use crate::utils::constants::*;

pub fn parse_method(class_file_name: &str, major_version: u16, is_interface: bool, pool: &ConstantPool, buf: &mut Bytes) -> Method {
    let mut access_flags = buf.get_u16();
    let name_index = buf.get_u16();
    let name = pool.get_string(name_index as usize)
        .expect(&format!("Invalid method in class_file file {}! Expected name index {} to be in \
            constant pool!", class_file_name, name_index))
        .clone();
    let descriptor_index = buf.get_u16();
    let descriptor = pool.get_string(descriptor_index as usize)
        .and_then(|value| MethodType::parse(value))
        .expect(&format!("Invalid descriptor for method in class file {}!", class_file_name));

    let mut other_flags: u8 = 0;
    if name == CLASS_INITIALIZER_METHOD_NAME {
        other_flags |= METHOD_IS_CONSTRUCTOR;
        if major_version < JAVA_VERSION_7 {
            access_flags = ACC_STATIC;
        } else if (access_flags & ACC_STATIC) == ACC_STATIC {
            let extra_flag = if major_version <= JAVA_VERSION_16 {
                ACC_STRICT
            } else {
                0
            };
            access_flags &= ACC_STATIC | extra_flag;
        } else {
            panic!("Invalid static initializer method ({}) in class_file file {}! Must be static!", CLASS_INITIALIZER_METHOD_NAME, class_file_name);
        }
    } else {
        verify_method_flags(class_file_name, major_version, access_flags, is_interface, &name);
    }
    if name == OBJECT_INITIALIZER_METHOD_NAME {
        other_flags |= METHOD_IS_STATIC_INITIALIZER;
        assert!(!is_interface, "Invalid method in class_file file {}! Interface cannot have a constructor!", class_file_name);
    }

    let attribute_count = buf.get_u16();
    let (code, checked_exception_indices, parameters, generic_signature) = parse_attributes(
        class_file_name,
        major_version,
        pool,
        buf,
        access_flags,
        attribute_count
    );
    Method::new(name, descriptor, generic_signature, access_flags, parameters, code, checked_exception_indices, other_flags)
}

fn parse_attributes(
    class_file_name: &str,
    major_version: u16,
    pool: &ConstantPool,
    buf: &mut Bytes,
    access_flags: u16,
    mut attribute_count: u16
) -> (Option<CodeBlock>, Vec<u16>, Vec<MethodParameter>, Option<String>) {
    let mut code = None;
    let mut checked_exception_indices = Vec::new();
    let mut parameters = Vec::new();
    let mut generic_signature = None;

    while attribute_count > 0 {
        assert!(buf.len() >= 6, "Truncated method attributes for method in class_file file {}!", class_file_name);
        let attribute_name_index = buf.get_u16();
        let attribute_length = buf.get_u32();
        let attribute_name = pool.get_utf8(attribute_name_index as usize)
            .expect(&format!("Invalid method attribute index {} in class_file file {}! Expected name \
                to be in constant pool!", attribute_name_index, class_file_name));

        if attribute_name == TAG_CODE {
            assert!(code.is_none(), "Expected single code attribute for method in class_file file {}!", class_file_name);
            assert!(access_flags & ACC_NATIVE == 0 && access_flags & ACC_ABSTRACT == 0, "Invalid code attribute \
                for method in class_file file {}! Abstract and native methods must not have code attributes!", class_file_name);
            code = Some(parse_code(class_file_name, pool, buf));
        } else if attribute_name == TAG_EXCEPTIONS {
            assert!(checked_exception_indices.is_empty(), "Expected single exceptions attribute for method in \
                class_file file {}!", class_file_name);
            let number_of_exceptions = buf.get_u16();
            for _ in 0..number_of_exceptions {
                checked_exception_indices.push(buf.get_u16());
            }
        } else if attribute_name == TAG_METHOD_PARAMETERS {
            assert!(parameters.is_empty(), "Expected single method parameters attribute for method in \
                class_file file {}!", class_file_name);
            let count = buf.get_u16();
            for _ in 0..count {
                parameters.push(MethodParameter::parse(class_file_name, pool, buf));
            }
        } else if attribute_name == TAG_SYNTHETIC {
            assert_eq!(attribute_length, 0, "Invalid synthetic attribute length {} for method in class_file \
                file {}!", attribute_length, class_file_name);
        } else if attribute_name == TAG_DEPRECATED {
            assert_eq!(attribute_length, 0, "Invalid deprecated attribute length {} for method in class_file \
                file {}!", attribute_length, class_file_name);
        } else if major_version >= JAVA_VERSION_1_5 {
            if attribute_name == TAG_SIGNATURE {
                assert!(generic_signature.is_none(), "Duplicate generic signature attribute found \
                    for method in class_file file {}!", class_file_name);
                generic_signature = parse_generic_signature(class_file_name, pool, attribute_length, buf, "method");
            }
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attribute_count -= 1;
    }
    (code, checked_exception_indices, parameters, generic_signature)
}

fn parse_code(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> CodeBlock {
    let max_stack = buf.get_u16();
    let max_locals = buf.get_u16();
    let code_length = buf.get_u32();
    let mut code = Vec::with_capacity(code_length as usize);
    for _ in 0..code_length {
        code.push(buf.get_u8());
    }

    // Parse exception handler table
    let exception_table_length = buf.get_u16();
    let mut exception_handlers = Vec::with_capacity(exception_table_length as usize);
    for _ in 0..exception_table_length {
        exception_handlers.push(ExceptionHandlerBlock::parse(buf));
    }

    // Parse attributes
    let attribute_count = buf.get_u16();
    let (line_number_table, local_variable_table, local_variable_type_table, stack_map_table) = parse_code_attributes(
        class_file_name,
        attribute_count,
        pool,
        buf
    );
    CodeBlock::new(
        max_stack,
        max_locals,
        code,
        exception_handlers,
        line_number_table,
        local_variable_table,
        local_variable_type_table,
        stack_map_table
    )
}

fn parse_code_attributes(
    class_file_name: &str,
    mut attribute_count: u16,
    pool: &ConstantPool,
    buf: &mut Bytes
) -> (Option<HashMap<u16, u16>>, Option<LocalVariableTable>, Option<LocalVariableTable>, Option<StackMapTable>) {
    let mut line_number_table = None;
    let mut local_variable_table = None;
    let mut local_variable_type_table = None;
    let mut stack_map_table = None;

    while attribute_count > 0 {
        assert!(buf.len() > 6, "Truncated code attributes for method in class_file file {}!", class_file_name);
        let attribute_name_index = buf.get_u16();
        let attribute_length = buf.get_u32();
        let attribute_name = pool.get_utf8(attribute_name_index as usize)
            .expect(&format!("Invalid code attribute index {} in class_file file {}! Expected name \
                to be in constant pool!", attribute_name_index, class_file_name));

        if attribute_name == TAG_LINE_NUMBER_TABLE {
            let table_length = buf.get_u16();
            let mut table = HashMap::with_capacity(table_length as usize);
            for _ in 0..table_length {
                let start_pc = buf.get_u16();
                let line_number = buf.get_u16();
                table.insert(start_pc, line_number);
            }
            line_number_table = Some(table)
        } else if attribute_name == TAG_LOCAL_VARIABLE_TABLE {
            local_variable_table = parse_local_variable_table(class_file_name, pool, buf);
        } else if attribute_name == TAG_LOCAL_VARIABLE_TYPE_TABLE {
            local_variable_type_table = parse_local_variable_table(class_file_name, pool, buf);
        } else if attribute_name == TAG_STACK_MAP_TABLE {
            stack_map_table = Some(parse_stack_map_table(buf));
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attribute_count -= 1;
    }
    (line_number_table, local_variable_table, local_variable_type_table, stack_map_table)
}

fn parse_local_variable_table(class_file_name: &str, pool: &ConstantPool, buf: &mut Bytes) -> Option<LocalVariableTable> {
    let length = buf.get_u16();
    let mut table = Vec::with_capacity(length as usize);
    for _ in 0..length {
        table.push(LocalVariable::parse(class_file_name, pool, buf));
    }
    Some(table)
}

fn verify_method_flags(class_file_name: &str, major_version: u16, flags: u16, is_interface: bool, name: &str) {
    let is_public = (flags & ACC_PUBLIC) != 0;
    let is_private = (flags & ACC_PRIVATE) != 0;
    let is_protected = (flags & ACC_PROTECTED) != 0;
    let is_static = (flags & ACC_STATIC) != 0;
    let is_final = (flags & ACC_FINAL) != 0;
    let is_synchronized = (flags & ACC_SYNCHRONIZED) != 0;
    let is_bridge = (flags & ACC_BRIDGE) != 0;
    let is_native = (flags & ACC_NATIVE) != 0;
    let is_abstract = (flags & ACC_ABSTRACT) != 0;
    let is_strict = (flags & ACC_STRICT) != 0;
    let major_1_5_or_above = major_version >= JAVA_VERSION_1_5;
    let major_8_or_above = major_version >= JAVA_VERSION_8;
    let major_17_or_above = major_version >= JAVA_VERSION_17;
    let is_constructor = name == OBJECT_INITIALIZER_METHOD_NAME;

    let is_illegal;
    if is_interface {
        if major_8_or_above {
            is_illegal = (is_public == is_private) || // Methods can't be both public and private
                // None of these are allowed on interface methods
                (is_native || is_protected || is_final || is_synchronized) ||
                // Interface instance methods can't be private, static, or strict
                (is_abstract && (is_private || is_static || (!major_17_or_above && is_strict)));
        } else if major_1_5_or_above {
            // Interface instance methods must be public and abstract
            is_illegal = !is_public || is_private || is_protected || is_static || is_final ||
                is_synchronized || is_native || !is_abstract || is_strict;
        } else {
            is_illegal = !is_public || is_static || is_final || is_native || !is_abstract;
        }
    } else {
        is_illegal = has_illegal_visibility(flags) ||
            // Constructor methods are instance methods that must have bodies, must not be generated
            // bridge methods, and aren't final as the class_file' access determines the constructor's access.
            (is_constructor && (is_static || is_final || is_synchronized || is_native || is_abstract ||
                (major_1_5_or_above && is_bridge))) ||
            // Abstract methods must be overridable by subclasses, and so none of these would make sense.
            (is_abstract && (is_final || is_native || is_private || is_static ||
                (major_1_5_or_above && (is_synchronized || (!major_17_or_above && is_strict)))));
    }

    assert!(!is_illegal, "Invalid method in class_file file {}! Access modifiers {} are illegal!", class_file_name, flags);
}

fn has_illegal_visibility(flags: u16) -> bool {
    let is_public = (flags & ACC_PUBLIC) != 0;
    let is_protected = (flags & ACC_PROTECTED) != 0;
    let is_private = (flags & ACC_PRIVATE) != 0;
    return (is_public && is_protected) || (is_public && is_private) || (is_protected && is_private)
}
