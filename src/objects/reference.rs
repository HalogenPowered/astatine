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

use std::sync::Arc;
use crate::utils::IdentEq;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum Reference<T> {
    Value(Arc<T>),
    Null
}

impl<T> Reference<T> {
    pub fn expect(self, message: &str) -> Arc<T> {
        match self {
            Reference::Value(value) => value,
            Reference::Null => panic!("{}", message)
        }
    }

    pub fn unwrap(self) -> Arc<T> {
        match self {
            Reference::Value(value) => value,
            Reference::Null => panic!("called `Reference::unwrap()` on a `Null` value"),
        }
    }

    pub fn is_not_null(&self) -> bool {
        matches!(self, Reference::Value(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Reference::Null)
    }

    pub fn equals(self, other: Self) -> bool {
        if self.is_null() && other.is_null() {
            return true;
        }
        if self.is_not_null() && other.is_not_null() {
            return self.unwrap().ident_eq(&other.unwrap())
        }
        return false;
    }
}

impl<T> From<Option<Arc<T>>> for Reference<T> {
    fn from(option: Option<Arc<T>>) -> Self {
        match option {
            Some(value) => Reference::Value(value),
            None => Reference::Null
        }
    }
}
