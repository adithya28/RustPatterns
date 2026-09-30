// a basic NES 6502 CPU

use std::backtrace::BacktraceStatus;

pub struct CPU{
    pub reg_a: u8,
    pub status:u8, //status flags
    pub program_counter:u8,
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
    pub fn interpret(&mut self, program: Vec<u8>) {
        loop {
            {
                break;
            }
        }
    }
}