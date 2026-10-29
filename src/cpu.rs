// a basic NES 6502 CPU

use std::backtrace::BacktraceStatus;
use std::ops::Deref;

use AddressingMode::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    NoneAddressing,
}

use crate::instruction_logic::{OPCODES};

const STACK: u16 = 0x0100;
const STACK_RESET: u8 = 0xfd;
pub struct CPU{
    pub reg_a: u8,
    pub status:u8, //status flags
    pub program_counter:u16,
    pub memory:[u8;65536],
    pub reg_x: u8,
    pub reg_y: u8,
    pub stack_pointer: u8,



}

impl CPU{

    pub fn new() -> Self {
        CPU {
            reg_a: 0,
            status: 0, //NV1B DIZC Bits 3 and 4 are irrelevant
            program_counter: 0,
            memory:[0;65536],
            reg_x:0,
            reg_y:0,
            stack_pointer:STACK_RESET,

        }

        }
    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.program_counter = 0x8000;
    }

    pub fn run(&mut self) {
        loop {

            let code = self.mem_read(self.program_counter);
            let temp_program_counter = self.program_counter;
            self.decode(code);
            self.program_counter += 1;
            print!("{} {}", self.program_counter, &code);
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
    pub fn decode(&mut self,code:u8)
    {
        let current_op = &(*OPCODES.get(&code).unwrap());
        (current_op.action_function)(self,&current_op.address_mode);
    }
    pub fn interpret(&mut self, program: Vec<u8>) {

    }
    pub(crate) fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub(crate) fn mem_write(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }



    pub(crate) fn read_u16(&self, addr: u16) -> u16 {
        let lo = self.mem_read(addr) as u16;
        let hi = self.mem_read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    pub(crate) fn reset(&mut self) {
        self.reg_a = 0;
        self.reg_x = 0;
        self.status = 0;
        self.reg_y = 0;
        self.program_counter = self.mem_read_u16(0xFFFC); // 6502 quirk at that specific address
        self.stack_pointer = STACK_RESET;
    }
    pub(crate) fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read((STACK as u16) + self.stack_pointer as u16)
    }

    pub(crate) fn stack_push(&mut self, data: u8) {
        self.mem_write((STACK as u16) + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1)
    }

    pub(crate) fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    pub(crate) fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;

        hi << 8 | lo
    }

}