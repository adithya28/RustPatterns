// a basic NES 6502 CPU

use std::backtrace::BacktraceStatus;
use crate::instruction_spec::AddressingMode;

pub struct CPU{
    pub reg_a: u8,
    pub status:u8, //status flags
    pub program_counter:u16,
    pub memory:[u8;65536],
    pub reg_x: u8,
    pub reg_y: u8,


}

impl CPU{

    pub fn new() -> Self {
        CPU {
            reg_a: 0,
            status: 0, //NV1B DIZC Bits 3 and 4 are irrelevant
            program_counter: 0,
            memory:[0;65536],
            reg_x:0,
            reg_y:0
        }

        }
    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.program_counter = 0x8000;
    }

    pub fn run(&mut self) {
        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match code {
                           _ => todo!()
            }
        }
    }
    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
    pub fn interpret(&mut self, program: Vec<u8>) {

    }
    pub fn reset(&mut self) {
        self.reg_a = 0;
        self.reg_x = 0;
        self.status = 0;
        self.reg_y = 0;
        self.program_counter = self.mem_read_u16(0xFFFC); // 6502 quirk at that specific address
    }


}