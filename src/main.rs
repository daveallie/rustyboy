#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", deny(clippy_pedantic))]
#![cfg_attr(feature="clippy", allow(missing_docs_in_private_items, similar_names, needless_range_loop))]
#![deny(missing_debug_implementations, missing_copy_implementations, trivial_casts, trivial_numeric_casts)]
#![deny(unsafe_code, unused_import_braces, unused_qualifications)]

// until I have logging
#![cfg_attr(feature="clippy", allow(print_stdout))]

extern crate glium;

mod clock;
mod cpu;
mod gpu;
mod input;
mod mbc;
mod mmu;
mod register;
mod screen;
mod serial;
mod sound;
#[cfg(feature = "debugger")]
mod debugger;

use cpu::CPU;
#[cfg(feature = "debugger")]
use debugger::Debugger;
use screen::Screen;
use std::{thread, env};
use std::sync::mpsc;

fn main() {
    let cart_path = match env::args().nth(1) {
        Some(v) => v,
        None => panic!("You must pass a cart path as the first argument!"),
    };

    let (screen_data_sender, screen_data_receiver) = mpsc::sync_channel(1);
    let (key_data_sender, key_data_receiver) = mpsc::channel();
    let (screen_exit_sender, screen_exit_receiver) = mpsc::channel();
    let (throttled_state_sender, throttled_state_receiver) = mpsc::channel();

    let cpu = CPU::new(
        &cart_path,
        screen_data_sender,
        key_data_receiver,
        throttled_state_receiver,
        screen_exit_receiver,
    );
    let screen = Screen::new(
        "Rustyboy",
        4,
        screen_data_receiver,
        key_data_sender,
        throttled_state_sender,
        screen_exit_sender,
    );

    run(cpu, screen);
}

#[cfg(not(feature = "debugger"))]
fn run(mut cpu: CPU, mut screen: Screen) {
    let cpu_thread = thread::spawn(move || { cpu.main_loop(); });

    screen.start_loop();
    if let Err(e) = cpu_thread.join() {
        panic!("Error: Failed to join CPU thread: {:?}", e);
    }
}

#[cfg(feature = "debugger")]
fn run(cpu: CPU, mut screen: Screen) {
    let debug_after_cycles = env::args().nth(2).map(|item| item.parse::<u32>().unwrap());
    let mut debugger = Debugger::new(debug_after_cycles, cpu);

    let cpu_thread = thread::spawn(move || loop {
        debugger.run();
    });

    screen.start_loop();
    if let Err(e) = cpu_thread.join() {
        panic!("Error: Failed to join CPU thread: {:?}", e);
    }
}
