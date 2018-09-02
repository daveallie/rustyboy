use cpu;
use register::Flags;
use std::io::{self, Write};
use std::process;

pub struct Debugger {
    current_steps: u32,
    debugging: bool,
    debug_after_cycles_enabled: bool,
    debug_after_cycles: u32,
    output: bool,
    cpu: cpu::CPU,
    reg_break_points: Vec<RegBreakPoint>,
}

struct RegBreakPoint {
    key: String,
    value: u32,
}

impl Debugger {
    pub fn new(debug_after_cycles: Option<u32>, cpu: cpu::CPU) -> Debugger {
        Debugger {
            current_steps: 0,
            debugging: false,
            debug_after_cycles_enabled: debug_after_cycles.is_some(),
            debug_after_cycles: debug_after_cycles.unwrap_or(0),
            output: true,
            cpu,
            reg_break_points: vec![],
        }
    }

    pub fn run(&mut self) {
        loop {
            if self.output {
                print!("{} ", self.current_steps);
                let addr = self.cpu.reg.pc;
                println!("instr: 0x{:X} -- opcode: 0x{:X}", addr, self.cpu.mmu.read_byte(addr));
            }
            self.cpu.run_cycle();
            self.current_steps += 1;
            if self.should_stop() {
                self.debug();
            }
        }
    }

    fn should_stop(&mut self) -> bool {
        if let Some(index) = self.stop_and_remove_breakon() {
            self.reg_break_points.remove(index);
            return true;
        }

        self.debug_after_cycles_enabled && self.current_steps >= self.debug_after_cycles
    }

    fn stop_and_remove_breakon(&self) -> Option<usize> {
        let register = self.cpu.reg;
        let break_points = &self.reg_break_points;

        for (i, break_point) in break_points.iter().enumerate() {
            let key = break_point.key.as_str();
            let reg_value = match key {
                "a" => register.a as u16,
                "b" => register.b as u16,
                "c" => register.c as u16,
                "d" => register.d as u16,
                "e" => register.e as u16,
                "h" => register.h as u16,
                "l" => register.l as u16,
                "pc" => register.pc,
                "sp" => register.sp,
                _ => self.cpu.mmu.read_byte(read_num(key) as u16) as u16,
            } as u32;

            if reg_value == break_point.value {
                output(&format!("Breaking as {} is {}\n", key, reg_value));
                return Some(i);
            }
        }

        None
    }

    fn debug(&mut self) {
        self.debugging = true;
        while self.debugging {
            self.read_input_and_process()
        }
    }

    fn read_input_and_process(&mut self) {
        let line = read_line();
        let mut words = line.trim().split(" ");
        match words.next() {
            Some("c") | Some("continue") => {
                self.debug_after_cycles_enabled = false;
                self.debugging = false;
            }
            Some("n") | Some("next") => {
                let jump = read_num(words.next().unwrap_or("0"));
                self.debug_after_cycles = self.current_steps + jump;
                self.debug_after_cycles_enabled = true;
                self.debugging = false;
            }
            Some("bo") | Some("breakon") => {
                let key = words.next().unwrap().to_owned();
                let value = read_num(words.next().unwrap_or("0"));
                self.reg_break_points.push(RegBreakPoint { key, value });
                self.debug_after_cycles_enabled = false;
                self.debugging = false;
            }
            Some("reg") | Some("registers") => {
                let action = words.next();

                if action.is_some() && action.unwrap() == "write" {
                    let reg_key = words.next().unwrap();
                    let reg_value = read_num(words.next().unwrap());

                    match reg_key {
                        "a" => self.cpu.reg.a = reg_value as u8,
                        "b" => self.cpu.reg.b = reg_value as u8,
                        "c" => self.cpu.reg.c = reg_value as u8,
                        "d" => self.cpu.reg.d = reg_value as u8,
                        "e" => self.cpu.reg.e = reg_value as u8,
                        "h" => self.cpu.reg.h = reg_value as u8,
                        "l" => self.cpu.reg.l = reg_value as u8,
                        "pc" => self.cpu.reg.pc = reg_value as u16,
                        "sp" => self.cpu.reg.sp = reg_value as u16,
                        _ => panic!("Unknown register {}\n", reg_key),
                    };
                } else {
                    let register = self.cpu.reg;
                    output("8 bit registers:\n");
                    output(&format!("a: 0x{:X}\n", register.a));
                    output(&format!("b: 0x{:X}\n", register.b));
                    output(&format!("c: 0x{:X}\n", register.c));
                    output(&format!("d: 0x{:X}\n", register.d));
                    output(&format!("e: 0x{:X}\n", register.e));
                    output(&format!("h: 0x{:X}\n", register.h));
                    output(&format!("l: 0x{:X}\n", register.l));

                    output("\n16 bit registers:\n");
                    output(&format!("pc: 0x{:X}\n", register.pc));
                    output(&format!("sp: 0x{:X}\n", register.sp));

                    output("\nFlags:\n");
                    output(&format!("Z: {}\n", register.get_flag(Flags::Z)));
                    output(&format!("N: {}\n", register.get_flag(Flags::N)));
                    output(&format!("H: {}\n", register.get_flag(Flags::H)));
                    output(&format!("C: {}\n", register.get_flag(Flags::C)));
                }
            }
            Some("lastbyte") => {
                output(&format!("0x{:X}\n", self.cpu.mmu.read_byte(self.cpu.reg.pc - 1)));
            }
            Some("rb") => {
                let addr = read_num(words.next().unwrap_or("0")) as u16;
                output(&format!("0x{:X}\n", self.cpu.mmu.read_byte(addr)));
            }
            Some("rw") => {
                let addr = read_num(words.next().unwrap_or("0")) as u16;
                output(&format!("0x{:X}\n", self.cpu.mmu.read_word(addr)));
            }
            Some("out") => {
                self.output = words.next().unwrap_or("on") == "on";
            }
            Some("dump") => self.dump(words.next().unwrap_or("mem_dump")),
            Some("exit") => process::exit(1),
            _ => output("Unknown command!\n"),
        };
    }

    fn dump(&self, folder_name: &str) {
        // wram
        // self.cpu.dump_wram();

        // zram
        // self.cpu.dump_zram();

        // oam
        // self.cpu.dump_oam(folder_name + "/oam.dmp");
    }
}

fn read_num(num_str: &str) -> u32 {
    if num_str.starts_with("0x") {
        u32::from_str_radix(&num_str[2..], 16).unwrap()
    } else {
        num_str.parse::<u32>().unwrap()
    }
}

fn read_line() -> String {
    output("What do you want?\n");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    buffer
}

fn output(line: &str) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write(line.as_bytes()).unwrap();
    handle.flush();
}
