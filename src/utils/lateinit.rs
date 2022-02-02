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

use std::cell::UnsafeCell;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use core::fmt::Error as FormatError;

pub struct LateInit<T>(UnsafeCell<Option<T>>);

unsafe impl<T> Sync for LateInit<T> {}

impl<T> LateInit<T> {
    pub const fn new() -> Self {
        LateInit(UnsafeCell::new(None))
    }

    pub fn init(&self, value: T) {
        assert!(self.option().is_none(), "LateInit.init called more than once!");
        unsafe { *self.0.get() = Some(value) }
    }

    pub fn get(&self) -> &T {
        match self.option() {
            Some(ref x) => x,
            _ => panic!("LateInit.get called before initialization!")
        }
    }

    fn option(&self) -> &Option<T> {
        unsafe { &*self.0.get() }
    }
}

impl<T> Deref for LateInit<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.get()
    }
}

impl<T> AsRef<T> for LateInit<T> {
    fn as_ref(&self) -> &T {
        self.get()
    }
}

macro_rules! impl_debug_display {
    ($T:ident) => {
        impl<T: $T> $T for LateInit<T> {
            fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
                match self.option() {
                    Some(ref x) => { x.fmt(f) },
                    None => { write!(f, "<UNINITIALIZED>") },
                }
            }
        }
    }
}

impl_debug_display!(Debug);
impl_debug_display!(Display);
