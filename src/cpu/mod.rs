mod ops;

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use register;
use mmu;
use input::Key;

pub struct CPU {
    pub reg: register::Registers,
    pub mmu: mmu::MMU,
    disable_interrupt_after: u8,
    enable_interrupt_after: u8,
    interrupts_enabled: bool,
    halting: bool,
}

impl CPU {
    pub const CLOCK_SPEED: u32 = 0x400_000_u32;
    pub const CYCLE_SPEED: u32 = Self::CLOCK_SPEED / 4;

    pub fn new(cart_path: &str, screen_data_sender: mpsc::Sender<Vec<u8>>, key_data_receiver: mpsc::Receiver<Key>) -> Self {
        Self {
            reg: register::Registers::new(),
            mmu: mmu::MMU::new(cart_path, screen_data_sender, key_data_receiver),
            disable_interrupt_after: 0,
            enable_interrupt_after: 0,
            interrupts_enabled: true,
            halting: false,
        }
    }

    pub fn main_loop(&mut self) {
        let mut time_before_run: Instant;
        let mut time_of_next_run: Instant = Instant::now();
        loop {
            time_before_run = Instant::now();
            if time_before_run < time_of_next_run {
                thread::sleep(time_of_next_run - time_before_run);
                time_before_run = Instant::now();
            }
            let completed_cycles = self.run_cycle();
            #[cfg_attr(feature="clippy", allow(cast_possible_truncation, cast_sign_loss))]
            let time_until_next_run = Duration::new(0,  (1_000_000_000_f64 * f64::from(completed_cycles) / f64::from(Self::CLOCK_SPEED)) as u32);
            time_of_next_run = time_before_run + time_until_next_run;
        }
    }

    pub fn run_cycle(&mut self) -> u8 {
        let cycles = self.run_cpu_cycle();
        self.mmu.run_cycle(cycles);
        cycles
    }

    pub fn step(&mut self) -> u8 {
        self.call_reg_op()
    }

    fn run_cpu_cycle(&mut self) -> u8 {
        self.update_interrupt_counters();
        let interrupt_cycles = self.jump_on_interrupt();
        if interrupt_cycles > 0 {
            return interrupt_cycles;
        }

        if self.halting {
            1 // noop
        } else {
            self.step()
        }
    }

    fn update_interrupt_counters(&mut self) {
        if self.disable_interrupt_after > 0 {
            self.disable_interrupt_after -= 1;
            if self.disable_interrupt_after == 0 {
                self.interrupts_enabled = false;
            }
        }

        if self.enable_interrupt_after > 0 {
            self.enable_interrupt_after -= 1;
            if self.enable_interrupt_after == 0 {
                self.interrupts_enabled = true;
            }
        }
    }

    fn jump_on_interrupt(&mut self) -> u8 {
        if !self.interrupts_enabled && !self.halting {
            return 0
        }

        let interrupt_flags = self.mmu.get_triggered_interrupts();
        if interrupt_flags == 0 {
            return 0
        }

        self.halting = false;
        if !self.interrupts_enabled {
            return 0
        }
        self.interrupts_enabled = false;

        let interrupt_jump_addresses: [u16; 5] = [0x40, 0x48, 0x50, 0x58, 0x60];

        for flag_number in 0..4 {
            let flag: u8 = 1 << flag_number;
            if interrupt_flags & flag > 0 {
                self.mmu.reset_interrupt(flag);
                let old_pc = self.reg.pc;
                self.push_stack(old_pc);
                self.reg.pc = interrupt_jump_addresses[flag_number];
                return 4;
            }
        }

        panic!("Unknown interrupt was not handled! 0b{:08b}", interrupt_flags);
    }

    fn get_byte(&mut self) -> u8 {
        let byte = self.mmu.read_byte(self.reg.pc);
        self.reg.pc += 1;
        byte
    }

    fn get_word(&mut self) -> u16 {
        let word = self.mmu.read_word(self.reg.pc);
        self.reg.pc += 2;
        word
    }

    fn push_stack(&mut self, val: u16) {
        self.reg.sp = self.reg.sp.wrapping_sub(2);
        self.mmu.write_word(self.reg.sp, val);
    }

    fn pop_stack(&mut self) -> u16 {
        let result = self.mmu.read_word(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(2);
        result
    }

    fn jr(&mut self) {
        #[cfg_attr(feature="clippy", allow(cast_possible_wrap))]
        let n = self.get_byte() as i8;
        #[cfg_attr(feature="clippy", allow(cast_possible_wrap, cast_possible_truncation, cast_sign_loss))]
        let new_pc = (u32::from(self.reg.pc) as i32 + i32::from(n)) as u16;
        self.reg.pc = new_pc;
    }
}
