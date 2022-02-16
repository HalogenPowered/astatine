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
use crate::constants::*;
use crate::utils::BufferExtras;

#[derive(Debug)]
pub struct StackMapTable {
    frames: Vec<StackMapFrame>
}

impl StackMapTable {
    pub(crate) fn parse(buf: &mut Bytes) -> Self {
        StackMapTable { frames: buf.get_generic_u16_array(StackMapFrame::parse) }
    }

    pub fn get(&self, index: usize) -> Option<&StackMapFrame> {
        self.frames.get(index)
    }
}

#[derive(Debug)]
pub struct StackMapFrame {
    frame_type: u8,
    offset_delta: u16,
    stack: Vec<VerificationType>,
    locals: Vec<VerificationType>
}

impl StackMapFrame {
    pub(crate) fn parse(buf: &mut Bytes) -> Self {
        let frame_type = buf.get_u8();
        let offset_delta;
        let mut stack = Vec::new();
        let mut locals = Vec::new();

        match frame_type {
            0..=63 => offset_delta = frame_type as u16,
            64..=127 => {
                offset_delta = (frame_type - 64) as u16;
                stack.push(VerificationType::parse(buf));
            },
            247 => {
                offset_delta = buf.get_u16();
                stack.push(VerificationType::parse(buf));
            },
            248..=250 => offset_delta = buf.get_u16(),
            251 => offset_delta = buf.get_u16(),
            252..=254 => {
                offset_delta = buf.get_u16();
                StackMapFrame::parse_types((frame_type - 251) as usize, &mut locals, buf);
            },
            255 => {
                offset_delta = buf.get_u16();
                StackMapFrame::parse_types(buf.get_u16() as usize, &mut locals, buf);
                StackMapFrame::parse_types(buf.get_u16() as usize, &mut stack, buf);
            },
            _ => panic!("Invalid stack map frame type {}!", frame_type)
        };
        StackMapFrame { frame_type, offset_delta, stack, locals }
    }

    #[inline]
    fn parse_types(count: usize, result: &mut Vec<VerificationType>, buf: &mut Bytes) {
        for _ in 0..count {
            result.push(VerificationType::parse(buf));
        }
    }

    pub fn frame_type(&self) -> u8 {
        self.frame_type
    }

    pub fn offset_delta(&self) -> u16 {
        self.offset_delta
    }

    pub fn get_stack(&self, index: usize) -> Option<&VerificationType> {
        self.stack.get(index)
    }

    pub fn get_locals(&self, index: usize) -> Option<&VerificationType> {
        self.locals.get(index)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct VerificationType {
    item: u8,
    offset: u16
}

impl VerificationType {
    fn parse(buf: &mut Bytes) -> Self {
        let item = buf.get_u8();
        let offset = if item == JVM_ITEM_OBJECT || item == JVM_ITEM_UNINITIALIZED { buf.get_u16() } else { 0 };
        VerificationType { item, offset }
    }

    pub fn item(&self) -> u8 {
        self.item
    }

    pub fn offset(&self) -> u16 {
        self.offset
    }
}
