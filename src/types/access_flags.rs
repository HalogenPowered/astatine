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

// Global
pub const ACC_SYNTHETIC: u16 = 0x1000;

// Classes, fields, methods, inner classes, and method parameters
pub const ACC_FINAL: u16 = 0x0010;

// Method parameters, modules, module requires, module exports, and module opens
pub const ACC_MANDATED: u16 = 0x8000;

// Classes, fields, methods, and inner classes
pub const ACC_PUBLIC: u16 = 0x0001;

// Classes, methods, and inner classes
pub const ACC_ABSTRACT: u16 = 0x0400;

// Classes, fields, and inner classes
pub const ACC_ENUM: u16 = 0x4000;

// Fields, methods, and inner classes
pub const ACC_PRIVATE: u16 = 0x0002;
pub const ACC_PROTECTED: u16 = 0x0004;
pub const ACC_STATIC: u16 = 0x0008;

// Classes and inner classes
pub const ACC_INTERFACE: u16 = 0x0200;
pub const ACC_ANNOTATION: u16 = 0x2000;

// Classes only
pub const ACC_SUPER: u16 = 0x0020;
pub const ACC_MODULE: u16 = 0x8000;

// Fields only
pub const ACC_VOLATILE: u16 = 0x0040;
pub const ACC_TRANSIENT: u16 = 0x0080;

// Methods only
pub const ACC_SYNCHRONIZED: u16 = 0x0020;
pub const ACC_BRIDGE: u16 = 0x0040;
pub const ACC_VARARGS: u16 = 0x0080;
pub const ACC_NATIVE: u16 = 0x0100;
pub const ACC_STRICT: u16 = 0x0800;

// Modules only
pub const ACC_OPEN: u16 = 0x0020;

// Module requires only
pub const ACC_TRANSITIVE: u16 = 0x0020;
pub const ACC_STATIC_PHASE: u16 = 0x0040;

pub const ALL_CLASS_MODIFIERS: u16 = ACC_PUBLIC | ACC_FINAL | ACC_SUPER | ACC_INTERFACE |
    ACC_ABSTRACT | ACC_SYNTHETIC | ACC_ANNOTATION | ACC_ENUM;
pub const ALL_CLASS_MODIFIERS_J9: u16 = ALL_CLASS_MODIFIERS | ACC_MODULE;

pub trait Accessible {
    fn flags(&self) -> u16;

    fn is_synthetic(&self) -> bool {
        self.flags() & ACC_SYNTHETIC != 0
    }
}
