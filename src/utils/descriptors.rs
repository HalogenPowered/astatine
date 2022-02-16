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

use std::fmt::{Display, Formatter};
use internship::IStr;
use nom::IResult;
use nom::branch::alt;
use nom::bytes::streaming::is_not;
use nom::character::streaming::{anychar, char};
use nom::combinator::{complete, fail, map};
use nom::multi::{fold_many_m_n, many0};
use nom::sequence::{delimited, pair, terminated};

#[derive(Debug, Eq, PartialEq, Clone)]
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

impl Display for FieldDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldDescriptor")
            .field("base", &self.base)
            .field("array_dimensions", &self.array_dimensions)
            .finish()
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
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

impl Display for MethodDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MethodDescriptor")
            .field("parameters", &self.parameters)
            .field("return_type", &self.return_type)
            .finish()
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
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

#[derive(Debug, Eq, PartialEq, Clone)]
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

fn semi_terminated(input: &str) -> IResult<&str, &str> {
    terminated(is_not(";"), char(';'))(input)
}

fn parse_type(input: &str) -> IResult<&str, FieldType> {
    let (input, char) = anychar(input)?;
    match char {
        'B' => Ok((input, FieldType::Byte)),
        'C' => Ok((input, FieldType::Byte)),
        'D' => Ok((input, FieldType::Double)),
        'F' => Ok((input, FieldType::Float)),
        'I' => Ok((input, FieldType::Int)),
        'J' => Ok((input, FieldType::Long)),
        'S' => Ok((input, FieldType::Short)),
        'Z' => Ok((input, FieldType::Boolean)),
        'L' => map(semi_terminated, |value| FieldType::Reference(IStr::new(value)))(input),
        _ => fail(input)
    }
}

fn parse_field(input: &str) -> IResult<&str, FieldDescriptor> {
    map(
        pair(fold_many_m_n(0, 255, char('['), || 0u8, |value, _| value + 1), parse_type),
        |value: (u8, FieldType)| FieldDescriptor::new(value.1, value.0)
    )(input)
}

fn parse_return(input: &str) -> IResult<&str, Option<FieldDescriptor>> {
    alt((
        map(parse_field, |value| Some(value)),
        map(char('V'), |_| None)
    ))(input)
}

fn parse_method(input: &str) -> IResult<&str, MethodDescriptor> {
    map(
        pair(delimited(char('('), many0(parse_field), char(')')), parse_return),
        |value| MethodDescriptor::new(value.0, value.1)
    )(input)
}

#[cfg(test)]
mod tests {
    use internship::IStr;
    use super::{FieldDescriptor, FieldType, MethodDescriptor};

    #[test]
    fn fields() {
        assert_eq!(
            FieldDescriptor::parse("[[[Lfoo bar net;"),
            Some(FieldDescriptor::new(FieldType::Reference(IStr::new("foo bar net")), 3))
        );
    }

    #[test]
    fn methods() {
        assert_eq!(
            MethodDescriptor::parse("([B[[LFoo;I)[LNetwork;"),
            Some(MethodDescriptor::new(
                vec![
                    FieldDescriptor::new(FieldType::Byte, 1),
                    FieldDescriptor::new(FieldType::Reference(IStr::new("Foo")), 2),
                    FieldDescriptor::new(FieldType::Int, 0)
                ],
                Some(FieldDescriptor::new(FieldType::Reference(IStr::new("Network")), 1))
            ))
        );
        assert_eq!(
            MethodDescriptor::parse("([B[[LFoo;I)V"),
            Some(MethodDescriptor::new(
                vec![
                    FieldDescriptor::new(FieldType::Byte, 1),
                    FieldDescriptor::new(FieldType::Reference(IStr::new("Foo")), 2),
                    FieldDescriptor::new(FieldType::Int, 0)
                ],
                None
            ))
        );
    }
}
