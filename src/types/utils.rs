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

macro_rules! named {
    () => {
        pub fn name(&self) -> &str {
            self.name.as_str()
        }
    }
}

macro_rules! describable {
    ($descriptor:ident) => {
        pub fn descriptor(&self) -> &crate::utils::descriptors::$descriptor {
            &self.descriptor
        }
    }
}

macro_rules! optional_string {
    ($name:ident) => {
        pub fn $name(&self) -> Option<&str> {
            self.$name.as_ref().map(|value| value.as_str())
        }
    }
}

macro_rules! generic {
    () => {
        optional_string!(generic_signature);
    }
}

macro_rules! versioned {
    () => {
        optional_string!(version);
    }
}
