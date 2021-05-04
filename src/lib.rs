mod bf_ops;
mod code_generator;

extern crate wabt;

use crate::bf_ops::parse;
use crate::code_generator::generate_wat;
use std::io::BufReader;
use wabt::wat2wasm;

pub fn compile(filepath: &str) -> Vec<u8> {
    let file = std::fs::File::open(filepath).expect(&format!("Cannot read file: {}", filepath));
    let mut reader = BufReader::new(file);
    let ops = parse(&mut reader);
    let wat = generate_wat(&ops);
    // println!("{}", wat);
    wat2wasm(wat).unwrap()
}
