pub struct Annotation {
    pub type_index: u16,
    pub element_values: Vec<ElementValuePair>
}

pub struct TypeAnnotation {
    pub target_type: u8,
    pub target_info: TypeAnnotationTarget,
    pub target_path: Vec<TypePath>,
    pub type_index: u16,
    pub element_values: Vec<ElementValuePair>
}

pub enum TypeAnnotationTarget {
    TypeParameter { index: u8 },
    SuperType { index: u16 },
    TypeParameterBound { index: u8, bound_index: u8 },
    Empty,
    FormalParameter { index: u8 },
    Throws { type_index: u16 },
    LocalVar { table: Vec<LocalVar> },
    Catch { exception_table_index: u16 },
    Offset { offset: u16 },
    TypeArgument { offset: u16, index: u8 }
}

pub struct TypePath {
    pub kind: u8,
    pub type_argument_index: u8
}

pub type ParameterAnnotation = Vec<Annotation>;

pub struct ElementValuePair {
    pub name_index: u16,
    pub value: ElementValue
}

pub struct ElementValue {
    pub tag: u8,
    pub value: ElementValueData
}

pub enum ElementValueData {
    ConstValueIndex(u16),
    ConstValue { type_name_index: u16, const_name_index: u16 },
    ClassInfoIndex(u16),
    Annotation { value: Annotation },
    Array { values: Vec<ElementValue> }
}

pub struct  LocalVar {
    pub start_pc: u16,
    pub length: u16,
    pub index: u16
}
