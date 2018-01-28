use std::env;

mod cpu;
mod mmu;
mod register;

fn main() {
    let cart_path = env::args().nth(1).unwrap();
    let mut cpu = cpu::CPU::new(&cart_path);

    loop {
        cpu.step();
    }
}
