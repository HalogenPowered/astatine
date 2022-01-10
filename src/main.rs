use std::io;
use crate::class_file::class_reader::parse_class;

pub mod class_file;
pub mod types;
pub mod utils;

fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("Expected input!");
    let input = buffer.trim_end();
    println!("{}", input);
    parse_class(&input);
}
