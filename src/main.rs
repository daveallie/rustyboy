use std::env;
use std::fs::File;
use std::io::Read;

mod cpu;
mod register;

fn main() {
    let cart_path = env::args().nth(1).unwrap();
    let mut cart_data: Vec<u8> = Vec::new();
    load_cart(&cart_path, &mut cart_data);

    println!("ROM loaded from {}", &cart_path);

    let cpu = cpu::CPU::new();
}

fn load_cart(cart_path: &str, buffer: &mut Vec<u8>) {
    let mut file = File::open(cart_path).unwrap();
    file.read_to_end(buffer).unwrap();
}
