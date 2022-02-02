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

// Class, field, method, code, and record component attribute tags
//pub(crate) const TAG_RUNTIME_VISIBLE_TYPE_ANNOTATIONS: &str = "RuntimeVisibleTypeAnnotations";
//pub(crate) const TAG_RUNTIME_INVISIBLE_TYPE_ANNOTATIONS: &str = "RuntimeInvisibleTypeAnnotations";

// Class, field, method, and record attribute tags
pub(crate) const TAG_SIGNATURE: &str = "Signature";
//pub(crate) const TAG_RUNTIME_VISIBLE_ANNOTATIONS: &str = "RuntimeVisibleAnnotations";
//pub(crate) const TAG_RUNTIME_INVISIBLE_ANNOTATIONS: &str = "RuntimeInvisibleAnnotations";

// Class, field, and method attribute tags
pub(crate) const TAG_SYNTHETIC: &str = "Synthetic";
pub(crate) const TAG_DEPRECATED: &str = "Deprecated";

// Class only attribute tags
pub(crate) const TAG_SOURCE_FILE: &str = "SourceFile";
pub(crate) const TAG_INNER_CLASSES: &str = "InnerClasses";
//pub(crate) const TAG_ENCLOSING_METHOD: &str = "EnclosingMethod";
//pub(crate) const TAG_SOURCE_DEBUG_EXTENSION: &str = "SourceDebugExtension";
pub(crate) const TAG_BOOTSTRAP_METHODS: &str = "BootstrapMethods";
//pub(crate) const TAG_MODULE: &str = "Module";
//pub(crate) const TAG_MODULE_PACKAGES: &str = "ModulePackages";
//pub(crate) const TAG_MODULE_MAIN_CLASS: &str = "ModuleMainClass";
//pub(crate) const TAG_NEST_HOST: &str = "NestHost";
//pub(crate) const TAG_NEST_MEMBERS: &str = "NestMembers";
pub(crate) const TAG_RECORD: &str = "Record";
//pub(crate) const TAG_PERMITTED_SUBCLASSES: &str = "PermittedSubclasses";

// Field only attribute tags
pub(crate) const TAG_CONSTANT_VALUE: &str = "ConstantValue";

// Method only attribute tags
pub(crate) const TAG_CODE: &str = "Code";
pub(crate) const TAG_EXCEPTIONS: &str = "Exceptions";
//pub(crate) const TAG_RUNTIME_VISIBLE_PARAMETER_ANNOTATIONS: &str = "RuntimeVisibleParameterAnnotations";
//pub(crate) const TAG_RUNTIME_INVISIBLE_PARAMETER_ANNOTATIONS: &str = "RuntimeInvisibleParameterAnnotations";
//pub(crate) const TAG_ANNOTATION_DEFAULT: &str = "AnnotationDefault";
pub(crate) const TAG_METHOD_PARAMETERS: &str = "MethodParameters";

// Code only attribute tags
pub(crate) const TAG_LINE_NUMBER_TABLE: &str = "LineNumberTable";
pub(crate) const TAG_LOCAL_VARIABLE_TABLE: &str = "LocalVariableTable";
pub(crate) const TAG_LOCAL_VARIABLE_TYPE_TABLE: &str = "LocalVariableTypeTable";
pub(crate) const TAG_STACK_MAP_TABLE: &str = "StackMapTable";
