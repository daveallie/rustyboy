mod ops;

use input::Key;
use mmu;
use register;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub struct CPU {
    pub reg: register::Registers,
    pub mmu: mmu::MMU,
    disable_interrupt_after: u8,
    enable_interrupt_after: u8,
    interrupts_enabled: bool,
    halting: bool,
    screen_exit_receiver: mpsc::Receiver<()>,
    throttled_state_receiver: mpsc::Receiver<bool>,
    throttled: bool,
}

impl CPU {
    const CLOCK_SPEED: u32 = 0x400_000_u32; // 4_194_304
    pub const CYCLE_SPEED: u32 = Self::CLOCK_SPEED / 4; // 1_048_576 = 1MHz
    const ADJUST_SPEED_EVERY_N_CYCLES: u32 = Self::CYCLE_SPEED / 64; // 8_192

    pub fn new(
        cart_path: &str,
        screen_data_sender: mpsc::SyncSender<Vec<u8>>,
        key_data_receiver: mpsc::Receiver<Key>,
        throttled_state_receiver: mpsc::Receiver<bool>,
        screen_exit_receiver: mpsc::Receiver<()>,
    ) -> Self {
        Self {
            reg: register::Registers::new(),
            mmu: mmu::MMU::new(cart_path, screen_data_sender, key_data_receiver),
            disable_interrupt_after: 0,
            enable_interrupt_after: 0,
            interrupts_enabled: true,
            halting: false,
            screen_exit_receiver,
            throttled_state_receiver,
            throttled: true,
        }
    }

    pub fn main_loop(&mut self) {
        let time_for_n_cycles = Duration::new(
            0,
            (1_000_000_000_f64 * f64::from(Self::ADJUST_SPEED_EVERY_N_CYCLES) / f64::from(Self::CYCLE_SPEED)) as u32,
        );

        let mut cycles_since_sleep: u32 = 0;
        let mut start_of_last_n_cycles = Instant::now();
        let mut cycles_since_last_log: u32 = 0;
        let mut time_of_next_log: Instant = Instant::now() + Duration::new(1, 0);
        loop {
            if cycles_since_sleep >= Self::ADJUST_SPEED_EVERY_N_CYCLES {
                if self.screen_exit_receiver.try_recv().is_ok() {
                    break;
                }
                if let Ok(v) = self.throttled_state_receiver.try_recv() {
                    self.throttled = v
                }

                if self.throttled {
                    let time_since_last_set_start = Instant::now() - start_of_last_n_cycles;
                    if time_since_last_set_start < time_for_n_cycles {
                        thread::sleep(time_for_n_cycles - time_since_last_set_start);
                    }
                    start_of_last_n_cycles = Instant::now();
                }
                cycles_since_sleep = 0;
            }

            if time_of_next_log <= Instant::now() {
                println!(
                    "RUNNING AT {}%",
                    100_f64 * f64::from(cycles_since_last_log) / f64::from(Self::CYCLE_SPEED)
                );
                time_of_next_log = Instant::now() + Duration::new(1, 0);
                cycles_since_last_log = 0;
            }

            let completed_cycles = u32::from(self.run_cycle());
            cycles_since_last_log += completed_cycles;
            cycles_since_sleep += completed_cycles;
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
            return 0;
        }

        let interrupt_flags = self.mmu.get_triggered_interrupts();
        if interrupt_flags == 0 {
            return 0;
        }

        self.halting = false;
        if !self.interrupts_enabled {
            return 0;
        }
        self.interrupts_enabled = false;

        let interrupt_jump_addresses: [u16; 5] = [0x40, 0x48, 0x50, 0x58, 0x60];

        for (flag_number, interrupt_jump_address) in interrupt_jump_addresses.iter().enumerate() {
            let flag = 1 << (flag_number as u8);
            if interrupt_flags & flag > 0 {
                self.mmu.reset_interrupt(flag);
                let old_pc = self.reg.pc;
                self.push_stack(old_pc);
                self.reg.pc = *interrupt_jump_address;
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

    fn get_signed_byte(&mut self) -> i8 {
        self.get_byte() as i8
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
        let n = self.get_byte() as i8;
        let new_pc = (u32::from(self.reg.pc) as i32 + i32::from(n)) as u16;
        self.reg.pc = new_pc;
    }
}
