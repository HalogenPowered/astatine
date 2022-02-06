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

use std::io;
use std::sync::Arc;
use crate::class_file::ClassLoader;
use crate::types::Class;

pub mod class_file;
pub mod types;
pub mod utils;
pub mod code;
pub mod objects;

fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("Expected input!");
    let input = buffer.trim_end();
    println!("{}", input);
    let loader = Arc::new(ClassLoader::new());
    let class = Arc::new(Class::parse(loader, input)).initialize();
    println!("{:#?}", class);
    println!("{}", class.is_public());
}
