// a basic NES 6502 CPU

use std::backtrace::BacktraceStatus;

pub struct CPU{
    pub reg_a: u8,
    pub status:u8, //status flags
    pub program_counter:u16,
    pub memory:[u8;65536]

}

impl CPU{

    pub fn new() -> Self {
        CPU {
            reg_a: 0,
            status: 0,
            program_counter: 0,
            memory:[0;65536]
        }

        }
    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.run()
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.program_counter = 0x8000;
    }
    pub fn run(&mut self) {
        loop {
            {
                break;
            }
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {

    }

}