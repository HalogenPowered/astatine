use std::io;
use crate::class_file::class_reader::parse_class;

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
    let class = parse_class(&input);
    println!("{:?}", class);
}
