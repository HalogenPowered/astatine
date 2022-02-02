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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use internship::IStr;
use crate::types::Class;

// TODO: Maybe locking the entire map with a single lock for reading and writing
//  isn't the greatest idea?
#[derive(Debug)]
pub struct ClassLoader {
    classes: Mutex<HashMap<IStr, Arc<Class>>>
}

impl ClassLoader {
    pub fn new() -> Self {
        ClassLoader { classes: Mutex::new(HashMap::new()) }
    }

    pub fn get_class(&self, name: &str) -> Option<Arc<Class>> {
        self.classes.lock()
            .unwrap()
            .get(name)
            .map(|value| Arc::clone(value))
    }

    pub fn load_class(self: Arc<ClassLoader>, name: &str) -> Arc<Class> {
        Arc::clone(self.classes.lock()
            .unwrap()
            .entry(IStr::new(name))
            .or_insert_with(|| Arc::new(Class::parse(Arc::clone(&self), name))))
    }
}
