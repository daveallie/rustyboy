use std::sync::mpsc;

use register;
use mmu;

mod ops;

const CLOCK_SPEED: f64 = 4194304f64;

pub struct CPU {
    reg: register::Registers,
    mmu: mmu::MMU,
    disable_interrupt: u32,
}

impl CPU {
    pub fn new(cart_path: &str, screen_data_sender: mpsc::SyncSender<Vec<u8>>) -> CPU {
        CPU {
            reg: register::Registers::new(),
            mmu: mmu::MMU::new(cart_path, screen_data_sender),
            disable_interrupt: 0,
        }
    }

    pub fn run_cycle(&mut self) -> u8 {
        let cycles = self.step();
        self.mmu.run_cycle(cycles);
        cycles
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
        self.mmu.write_word(self.reg.sp, val);
        self.reg.sp = self.reg.sp.wrapping_sub(2);
    }

    fn pop_stack(&mut self) -> u16 {
        let result = self.mmu.read_word(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(2);
        result
    }

    fn jr(&mut self) {
        let n = self.get_byte() as i8;
        self.reg.pc = ((self.reg.pc as u32 as i32) + (n as i32)) as u16;
    }
}

//struct Clock {
//
//}
//
//impl Clock {
//    fn new() -> Clock {
//        Clock {
//            m
//        }
//    }
//}
