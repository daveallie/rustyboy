#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", deny(clippy_pedantic))]
#![cfg_attr(feature="clippy", allow(missing_docs_in_private_items))]

#![deny(missing_debug_implementations, missing_copy_implementations, trivial_casts, trivial_numeric_casts, unsafe_code, unused_import_braces, unused_qualifications)]

extern crate glium;
extern crate rand;

use std::{thread, env};
use std::sync::mpsc;

mod cpu;
mod gpu;
mod mmu;
mod register;
mod screen;
mod serial;
#[cfg(feature = "debugger")]
mod debugger;

fn main() {
    let cart_path = env::args().nth(1).unwrap();

    let (screen_data_sender, screen_data_receiver) = mpsc::sync_channel(1);

    let cpu = cpu::CPU::new(&cart_path, screen_data_sender);
    let screen = screen::Screen::new("Rustyboy", 4, screen_data_receiver);

    run(cpu, screen);
}

#[cfg(not(feature = "debugger"))]
fn run(mut cpu: cpu::CPU, mut screen: screen::Screen) {
    let cpu_thread = thread::spawn(move || {
        loop {
            cpu.run_cycle();
        }
    });

    screen.start_loop();
    cpu_thread.join().unwrap();
}

#[cfg(feature = "debugger")]
fn run(mut cpu: cpu::CPU) {
    let debug_after_cycles = env::args().nth(2).map(|item| item.parse::<u32>().unwrap());
    let mut debugger = debugger::Debugger::new(debug_after_cycles, cpu);
    debugger.run();
}
