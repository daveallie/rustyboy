use std::env;

mod cpu;
mod gpu;
mod mmu;
mod register;
mod serial;
#[cfg(feature = "debugger")]
mod debugger;

fn main() {
    let cart_path = env::args().nth(1).unwrap();
    let mut cpu = cpu::CPU::new(&cart_path);

    run(cpu);
}

#[cfg(not(feature = "debugger"))]
fn run(mut cpu: cpu::CPU) {
    loop {
        cpu.step();
    }
}

#[cfg(feature = "debugger")]
fn run(mut cpu: cpu::CPU) {
    let debug_after_cycles = env::args().nth(2).map(|item| item.parse::<u32>().unwrap());
    let mut debugger = debugger::Debugger::new(debug_after_cycles, cpu);
    debugger.run();
}
