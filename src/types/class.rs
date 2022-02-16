/*
 * Copyright (C) 2022 Callum Seabrook <callum.seabrook@prevarinite.com>
 *
 * This program is free software; you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation; version 2.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program; if not, write to the Free Software Foundation, Inc., 51 Franklin
 * Street, Fifth Floor, Boston, MA 02110-1301, USA.
 */

use bytes::{Buf, Bytes};
use internship::IStr;
use std::fs;
use std::ops::Deref;
use std::sync::Arc;
use astatine_macros::{Nameable, accessible};
use crate::class_file::ClassLoader;
use crate::constants::*;
use crate::types::method::BootstrapMethod;
use crate::utils::{BufferExtras, IdentEq};
use crate::utils::constants::JAVA_LANG_OBJECT_NAME;
use super::access_flags::*;
use super::ConstantPool;
use super::field::Field;
use super::method::Method;
use super::RecordComponent;

#[accessible(final, public, abstract, interface)]
#[derive(Debug, Nameable)]
pub struct Class {
    loader: Arc<ClassLoader>,
    minor_version: u16,
    major_version: u16,
    access_flags: AccessFlags,
    constant_pool: ConstantPool,
    name: IStr,
    super_class: Option<Arc<Class>>,
    interfaces: Vec<Arc<Class>>,
    fields: Vec<Arc<Field>>,
    methods: Vec<Arc<Method>>,
    source_file_name: Option<IStr>,
    inner_classes: Vec<InnerClassInfo>,
    record_components: Vec<RecordComponent>,
    bootstrap_methods: Vec<Arc<BootstrapMethod>>
}

impl Class {
    pub(crate) fn parse(loader: Arc<ClassLoader>, file_name: &str) -> Self {
        let contents = fs::read(file_name)
            .expect(&format!("Class file name {} could not be read!", file_name));
        let mut buf = Bytes::from(contents);
        let magic = buf.get_u32();
        assert_eq!(magic, JAVA_CLASS_FILE_MAGIC, "Invalid class file magic header! Expected {}, \
            got {}!", JAVA_CLASS_FILE_MAGIC, magic);

        let minor_version = buf.get_u16();
        let major_version = buf.get_u16();
        let constant_pool = ConstantPool::parse(&mut buf);

        let mut access_flags = if major_version >= JAVA_VERSION_9 {
            (buf.get_u16() as u32) & (JVM_RECOGNIZED_CLASS_MODIFIERS | JVM_ACC_MODULE)
        } else {
            (buf.get_u16() as u32) & JVM_RECOGNIZED_CLASS_MODIFIERS
        };
        if access_flags & JVM_ACC_INTERFACE != 0 && major_version < JAVA_VERSION_6 {
            // Set abstract flag for backwards compatibility
            access_flags |= JVM_ACC_ABSTRACT;
        }
        verify_modifiers(major_version, access_flags);
        let access_flags = AccessFlags::from(access_flags);

        let this_class = buf.get_u16();
        let name = constant_pool.get_class_name(this_class as usize)
            .expect(&format!("Invalid name for class file {}! Expected index {} to be in \
                constant pool!", file_name, this_class));
        let super_class = resolve_superclass(Arc::clone(&loader), name.as_str(), &constant_pool,
                                             buf.get_u16(), access_flags);

        let interfaces = buf.get_generic_u16_array(|buf| {
            let index = buf.get_u16();
            constant_pool.get_class_no_holder(index as usize, Arc::clone(&loader))
                .expect(&format!("Invalid class file {}! Expected super interface index {} to be \
                    in constant pool!", file_name, index))
        });
        let fields = buf.get_generic_u16_array(|buf| {
            Arc::new(Field::parse(&constant_pool, buf, major_version, access_flags))
        });
        let methods = buf.get_generic_u16_array(|buf| {
            Arc::new(Method::parse(Arc::clone(&loader), file_name, &constant_pool, buf, major_version, access_flags))
        });

        let attributes = parse_attributes(&constant_pool, &mut buf);
        assert_eq!(buf.remaining(), 0, "Extra bytes found in class file {}!", file_name);
        Class {
            loader,
            minor_version,
            major_version,
            access_flags,
            constant_pool,
            name,
            super_class,
            interfaces,
            fields,
            methods,
            source_file_name: attributes.0,
            inner_classes: attributes.1.unwrap_or(Vec::new()),
            record_components: attributes.2.unwrap_or(Vec::new()),
            bootstrap_methods: attributes.3.unwrap_or(Vec::new())
        }
    }

    pub(crate) fn initialize(self: Arc<Class>) -> Arc<Class> {
        self.constant_pool.set_holder(Arc::clone(&self));
        self
    }

    pub fn loader(&self) -> Arc<ClassLoader> {
        Arc::clone(&self.loader)
    }

    pub fn major_version(&self) -> u16 {
        self.major_version
    }

    pub fn minor_version(&self) -> u16 {
        self.minor_version
    }

    pub fn constant_pool(&self) -> &ConstantPool {
        &self.constant_pool
    }

    pub fn super_class(&self) -> Option<Arc<Class>> {
        self.super_class.as_ref().map(|value| Arc::clone(value))
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn source_file_name(&self) -> Option<&str> {
        self.source_file_name.as_ref().map(|value| value.as_str())
    }

    pub fn inner_classes(&self) -> &[InnerClassInfo] {
        self.inner_classes.as_slice()
    }

    pub fn record_components(&self) -> &[RecordComponent] {
        self.record_components.as_slice()
    }

    pub fn bootstrap_methods(&self) -> &[Arc<BootstrapMethod>] {
        self.bootstrap_methods.as_slice()
    }

    pub fn is_super(&self) -> bool {
        self.access_flags.value() & JVM_ACC_SUPER != 0
    }

    pub fn is_module(&self) -> bool {
        self.access_flags.value() & JVM_ACC_MODULE != 0
    }

    pub fn is_subclass(&self, other: &Self) -> bool {
        if self.ident_eq(other) {
            return true;
        }
        let mut super_class = self.super_class();
        while super_class.is_some() {
            let class = super_class.unwrap();
            if class.deref().ident_eq(other) {
                return true;
            }
            super_class = class.super_class();
        }
        false
    }
}

fn resolve_superclass(
    loader: Arc<ClassLoader>,
    name: &str,
    pool: &ConstantPool,
    index: u16,
    flags: AccessFlags
) -> Option<Arc<Class>> {
    assert!(flags.is_interface() || index != 0, "Invalid super class! Interfaces must always have an \
        explicit superclass!");
    if index == 0 {
        assert_eq!(name, JAVA_LANG_OBJECT_NAME, "Invalid super class! Every class other than {} must \
            have an explicit superclass of {} or one of its subclasses!", JAVA_LANG_OBJECT_NAME,
            JAVA_LANG_OBJECT_NAME);
        return None;
    }
    pool.get_class_no_holder(index as usize, loader)
}

#[accessible(final, public, abstract, private, protected, static, interface)]
#[derive(Debug)]
pub struct InnerClassInfo {
    index: u16,
    name: Option<IStr>,
    access_flags: AccessFlags,
    outer_index: u16
}

impl InnerClassInfo {
    pub(crate) fn parse(pool: &ConstantPool, buf: &mut Bytes) -> Self {
        let index = buf.get_u16();
        let outer_index = buf.get_u16();
        let name = pool.get_utf8(buf.get_u16() as usize);
        let access_flags = AccessFlags::from(buf.get_u16());
        InnerClassInfo { index, name, access_flags, outer_index }
    }

    pub fn index(&self) -> u16 {
        self.index
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(IStr::as_str)
    }

    pub fn outer_index(&self) -> u16 {
        self.outer_index
    }
}

type ClassAttributes = (Option<IStr>, Option<Vec<InnerClassInfo>>, Option<Vec<RecordComponent>>,
                        Option<Vec<Arc<BootstrapMethod>>>);

fn parse_attributes(pool: &ConstantPool, buf: &mut Bytes) -> ClassAttributes {
    let mut source_file_name = None;
    let mut inner_classes = None;
    let mut record_components = None;
    let mut bootstrap_methods = None;

    let mut attribute_count = buf.get_u16();
    while attribute_count > 0 {
        assert!(buf.len() >= 6, "Truncated class attributes!");
        let attribute_name = pool.get_utf8(buf.get_u16() as usize).unwrap();
        let attribute_length = buf.get_u32();

        if attribute_name == JVM_ATTRIBUTE_SOURCE_FILE {
            assert_eq!(attribute_length, 2, "Invalid source file attribute! Expected length of 2, \
                was {}!", attribute_length);
            assert!(source_file_name.is_none(), "Duplicate source file attribute!");
            let source_file_index = buf.get_u16();
            let source_file = pool.get_utf8(buf.get_u16() as usize)
                .expect(&format!("Invalid source file attribute! Expected name index {} to be in \
                    constant pool!", source_file_index));
            source_file_name = Some(source_file);
        } else if attribute_name == JVM_ATTRIBUTE_INNER_CLASSES {
            assert!(inner_classes.is_none(), "Duplicate inner classes attribute!");
            let number_of_classes = buf.get_u16();
            let mut classes = Vec::with_capacity(number_of_classes as usize);
            for _ in 0..number_of_classes {
                classes.push(InnerClassInfo::parse(pool, buf));
            }
            inner_classes = Some(classes);
        } else if attribute_name == JVM_ATTRIBUTE_RECORD {
            assert!(record_components.is_none(), "Duplicate record attribute!");
            let components_count = buf.get_u16();
            let mut components = Vec::with_capacity(components_count as usize);
            for _ in 0..components_count {
                components.push(RecordComponent::parse(pool, buf));
            }
            record_components = Some(components);
        } else if attribute_name == JVM_ATTRIBUTE_BOOTSTRAP_METHODS {
            assert!(bootstrap_methods.is_none(), "Duplicate bootstrap methods attribute!");
            let methods_count = buf.get_u16();
            let mut methods = Vec::with_capacity(methods_count as usize);
            for _ in 0..methods_count {
                methods.push(Arc::new(BootstrapMethod::parse(pool, buf)));
            }
            bootstrap_methods = Some(methods)
        } else {
            // Skip past any attribute that we don't recognise
            buf.advance(attribute_length as usize);
        }
        attribute_count -= 1;
    }

    if pool.has_dynamic() {
        assert!(bootstrap_methods.is_some(), "Invalid class attributes! Bootstrap methods must be \
            present if the class file has a Dynamic or InvokeDynamic constant in the constant pool!");
    }
    (source_file_name, inner_classes, record_components, bootstrap_methods)
}

fn verify_modifiers(major_version: u16, flags: u32) {
    let is_module = flags & JVM_ACC_MODULE != 0;
    assert!(major_version >= JAVA_VERSION_9 || !is_module, "Invalid class modifiers! Module flag \
        should not be set for classes before Java 9!");
    assert!(!is_module, "Cannot load class as it is a module!");

    let is_final = flags & JVM_ACC_FINAL != 0;
    let is_super = flags & JVM_ACC_SUPER != 0;
    let is_interface = flags & JVM_ACC_INTERFACE != 0;
    let is_abstract = flags & JVM_ACC_ABSTRACT != 0;
    let is_annotation = flags & JVM_ACC_ANNOTATION != 0;
    let is_enum = flags & JVM_ACC_ENUM != 0;
    let major_1_5_or_above = major_version >= JAVA_VERSION_1_5;

    let is_illegal = (is_abstract && is_final) ||
        (is_interface && !is_abstract) ||
        (is_interface && major_1_5_or_above && (is_super || is_enum)) ||
        (!is_interface && major_1_5_or_above && is_annotation);
    assert!(!is_illegal, "Illegal class modifiers {}!", flags);
}

const JAVA_CLASS_FILE_MAGIC: u32 = 0xCAFEBABE;
