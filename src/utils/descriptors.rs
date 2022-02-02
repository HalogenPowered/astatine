use internship::IStr;
use nom::{named, terminated, is_not, char,
          switch, take, value, map,
          pair, fold_many_m_n, delimited, many0,
          alt, combinator::complete};

#[derive(Debug, Clone)]
pub struct FieldDescriptor {
    base: FieldType,
    array_dimensions: u8
}

impl FieldDescriptor {
    pub fn parse(input: &str) -> Option<Self> {
        complete(parse_field)(input).ok().map(|value| value.1)
    }

    pub fn new(base: FieldType, array_dimensions: u8) -> Self {
        FieldDescriptor { base, array_dimensions }
    }

    pub fn base(&self) -> &FieldType {
        &self.base
    }

    pub fn array_dimensions(&self) -> u8 {
        self.array_dimensions
    }
}

impl From<FieldType> for FieldDescriptor {
    fn from(value: FieldType) -> Self {
        FieldDescriptor::new(value, 0)
    }
}

#[derive(Debug, Clone)]
pub struct MethodDescriptor {
    parameters: Vec<FieldDescriptor>,
    return_type: Option<FieldDescriptor>
}

impl MethodDescriptor {
    pub fn parse(input: &str) -> Option<Self> {
        complete(parse_method)(input).ok().map(|value| value.1)
    }

    pub fn new(parameters: Vec<FieldDescriptor>, return_type: Option<FieldDescriptor>) -> Self {
        MethodDescriptor { parameters, return_type }
    }

    pub fn parameters(&self) -> &[FieldDescriptor] {
        self.parameters.as_slice()
    }

    pub fn return_type(&self) -> Option<&FieldDescriptor> {
        self.return_type.as_ref()
    }
}

#[derive(Debug, Clone)]
pub enum FieldType {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Reference(IStr),
    Short,
    Boolean
}

impl FieldType {
    pub fn parse(input: &str) -> Option<Self> {
        complete(parse_type)(input).ok().map(|value| value.1)
    }
}

pub enum Descriptor {
    Field(FieldDescriptor),
    Method(MethodDescriptor)
}

impl From<FieldDescriptor> for Descriptor {
    fn from(value: FieldDescriptor) -> Self {
        Descriptor::Field(value)
    }
}

impl From<MethodDescriptor> for Descriptor {
    fn from(value: MethodDescriptor) -> Self {
        Descriptor::Method(value)
    }
}

// The following parser functions are all from here:
// https://github.com/powerboat9/java-desc/blob/1b4e15fd9014962c9704ab69386c12898b44562b/src/lib.rs

named!(semi_terminated<&str, &str>, terminated!(is_not!(";"), char!(';')));

named!(parse_type<&str, FieldType>, switch!(take!(1),
    "B" => value!(FieldType::Byte) |
    "C" => value!(FieldType::Char) |
    "D" => value!(FieldType::Double) |
    "F" => value!(FieldType::Float) |
    "I" => value!(FieldType::Int) |
    "J" => value!(FieldType::Long) |
    "S" => value!(FieldType::Short) |
    "Z" => value!(FieldType::Boolean) |
    "L" => map!(semi_terminated, |value| FieldType::Reference(IStr::new(value))))
);

named!(parse_field<&str, FieldDescriptor>, map!(
    pair!(fold_many_m_n!(0, 255, char!('['), 0u8, |value, _| value + 1), parse_type),
    |value: (u8, FieldType)| FieldDescriptor { base: value.1, array_dimensions: value.0 }
));

named!(parse_return<&str, Option<FieldDescriptor>>, alt!(
    map!(parse_field, |v| Some(v)) |
    map!(char!('V'), |_| None)
));

named!(parse_method<&str, MethodDescriptor>, map!(
    pair!(delimited!(char!('('), many0!(parse_field), char!(')')), parse_return),
    |value| MethodDescriptor::new(value.0, value.1)
));
