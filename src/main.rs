use std::io;
use crate::class_file::class_loader::ClassLoader;
use crate::types::class::Class;

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
    let mut loader = ClassLoader::new();
    let class = Class::parse(&mut loader, &input);
    println!("{:#?}", class);
}
