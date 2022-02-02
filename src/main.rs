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
}
