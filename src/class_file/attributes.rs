use bytes::{Buf, Bytes};

use crate::class_file::annotations::*;
use crate::class_file::exceptions::ExceptionTable;
use crate::class_file::method::{BootstrapMethod, MethodParameter};
use crate::class_file::module::*;
use crate::class_file::record::RecordComponent;
use crate::class_file::stack_map_frames::StackMapFrame;

pub struct Attribute {
    pub name_index: u16,
    pub length: u32,
    pub data: AttributeData
}

impl Attribute {
    pub fn parse(buf: &mut Bytes) -> Self {
        let name_index = buf.get_u16();
        let length = buf.get_u32();
    }


}

pub enum AttributeData {
    ConstantValue { index: u16 },
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
        exception_table: ExceptionTable,
        attributes: Vec<Attribute>
    },
    StackMapTable { entries: Vec<StackMapFrame> },
    Exceptions { index_table: Vec<u16> },
    InnerClasses { classes: Vec<UnresolvedInnerClass> },
    EnclosingMethod { class_index: u16, method_index: u16 },
    Synthetic,
    Signature { index: u16 },
    SourceFile { index: u16 },
    SourceDebugExtension { debug_extension: Vec<u8> },
    LineNumberTable { entries: Vec<LineNumber> },
    LocalVariableTable { entries: Vec<LocalVariable> },
    LocalVariableTypeTable { entries: Vec<LocalVariableType> },
    Deprecated,
    RuntimeVisibleAnnotations { entries: Vec<Annotation> },
    RuntimeInvisibleAnnotations { entries: Vec<Annotation> },
    RuntimeVisibleParameterAnnotations { entries: Vec<ParameterAnnotation> },
    RuntimeInvisibleParameterAnnotations { entries: Vec<ParameterAnnotation> },
    RuntimeVisibleTypeAnnotations { entries: Vec<TypeAnnotation> },
    RuntimeInvisibleTypeAnnotations { entries: Vec<TypeAnnotation> },
    AnnotationDefault { value: ElementValue },
    BootstrapMethods { entries: Vec<BootstrapMethod> },
    MethodParameters { entries: Vec<MethodParameter> },
    Module {
        name_index: u16,
        flags: u16,
        version_index: u16,
        requirements: Vec<ModuleRequirement>,
        exports: Vec<ModuleExport>,
        openings: Vec<ModuleOpening>,
        uses: Vec<u16>,
        provided: Vec<ProvidedModule>
    },
    ModulePackages { indices: Vec<u16> },
    ModuleMainClass { index: u16 },
    NestHost { host_class_index: u16 },
    NestMembers { classes: Vec<u16> },
    Record { components: Vec<RecordComponent> },
    PermittedSubClasses { classes: Vec<u16> }
}

pub struct UnresolvedInnerClass {
    pub info_index: u16,
    pub outer_info_index: u16,
    pub name_index: u16,
    pub access_flags: u16
}

pub struct LineNumber {
    pub start_pc: u16,
    pub line_number: u16
}

pub struct LocalVariable {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub index: u16
}

pub struct LocalVariableType {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub signature_index: u16,
    pub index: u16
}
