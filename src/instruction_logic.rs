use std::collections::HashMap;
use std::sync::LazyLock;
use crate::cpu::CPU;

// ============================================================
// Status register: single u8, one bit per flag
// ============================================================

const CARRY: u8             = 0b0000_0001; // C  - bit 0
const ZERO: u8              = 0b0000_0010; // Z  - bit 1
const INTERRUPT_DISABLE: u8 = 0b0000_0100; // I  - bit 2
const DECIMAL: u8           = 0b0000_1000; // D  - bit 3
const BREAK: u8             = 0b0001_0000; // B  - bit 4
const UNUSED: u8            = 0b0010_0000; // U  - bit 5 (always 1 on real chip)
const OVERFLOW: u8          = 0b0100_0000; // V  - bit 6
const NEGATIVE: u8          = 0b1000_0000; // N  - bit 7

// Stack pointer lives at memory[0x0000] since the struct has no SP field.
const SP_ADDR: u16 = 0x0000;

use AddressingMode::*;
// ============================================================
// Primitives
// ============================================================

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

#[derive(Debug, Clone)]
pub struct OpCode {
    pub opcode_hex: u8,
    pub opcode_str: String,
    pub cycles: u8,
    pub bytes: u8,
    pub is_no_op: bool,
    pub address_mode: AddressingMode,
    pub action_function:fn(&mut CPU, &AddressingMode)
}

pub static OPCODES: LazyLock<HashMap<u8,OpCode>> = LazyLock::new(|| {
    use AddressingMode::*;

    fn op(h: u8, s: &str, c: u8, b: u8, n: bool, m: AddressingMode, mapped_function: fn(&mut CPU, &AddressingMode)) -> OpCode {
        OpCode {
            opcode_hex: h,
            opcode_str: s.to_string(),
            cycles: c,
            bytes: b,
            is_no_op: n,
            address_mode: m,
            action_function : mapped_function
        }
    }
    fn nop(h: u8) -> OpCode {
        op(h, "NOP", 2, 1, true, NoneAddressing,CPU::nop)
    }

    let instruction_vector = vec![
        // 0x00 – 0x0F
        op(0x00, "BRK", 7, 1, false, NoneAddressing, CPU::brk),
        op(0x01, "ORA", 6, 2, false, IndirectX, CPU::ora),
        nop(0x02),
        nop(0x03),
        nop(0x04),
        op(0x05, "ORA", 3, 2, false, ZeroPage, CPU::ora),
        op(0x06, "ASL", 5, 2, false, ZeroPage, CPU::asl),
        nop(0x07),
        op(0x08, "PHP", 3, 1, false, NoneAddressing, CPU::php),
        op(0x09, "ORA", 2, 2, false, Immediate, CPU::ora),
        op(0x0A, "ASL", 2, 1, false, NoneAddressing, CPU::asl),
        nop(0x0B),
        nop(0x0C),
        op(0x0D, "ORA", 4, 3, false, Absolute, CPU::ora),
        op(0x0E, "ASL", 6, 3, false, Absolute, CPU::asl),
        nop(0x0F),

        // 0x10 – 0x1F
        op(0x10, "BPL", 2, 2, false, NoneAddressing, CPU::bpl),
        op(0x11, "ORA", 5, 2, false, IndirectY, CPU::ora),
        nop(0x12),
        nop(0x13),
        nop(0x14),
        op(0x15, "ORA", 4, 2, false, ZeroPageX, CPU::ora),
        op(0x16, "ASL", 6, 2, false, ZeroPageX, CPU::asl),
        nop(0x17),
        op(0x18, "CLC", 2, 1, false, NoneAddressing, CPU::clc),
        op(0x19, "ORA", 4, 3, false, AbsoluteY, CPU::ora),
        nop(0x1A),
        nop(0x1B),
        nop(0x1C),
        op(0x1D, "ORA", 4, 3, false, AbsoluteX, CPU::ora),
        op(0x1E, "ASL", 7, 3, false, AbsoluteX, CPU::asl),
        nop(0x1F),

        // 0x20 – 0x2F
        op(0x20, "JSR", 6, 3, false, Absolute, CPU::jsr),
        op(0x21, "AND", 6, 2, false, IndirectX, CPU::and),
        nop(0x22),
        nop(0x23),
        op(0x24, "BIT", 3, 2, false, ZeroPage, CPU::bit),
        op(0x25, "AND", 3, 2, false, ZeroPage, CPU::and),
        op(0x26, "ROL", 5, 2, false, ZeroPage, CPU::rol),
        nop(0x27),
        op(0x28, "PLP", 4, 1, false, NoneAddressing, CPU::plp),
        op(0x29, "AND", 2, 2, false, Immediate, CPU::and),
        op(0x2A, "ROL", 2, 1, false, NoneAddressing, CPU::rol),
        nop(0x2B),
        op(0x2C, "BIT", 4, 3, false, Absolute, CPU::bit),
        op(0x2D, "AND", 4, 3, false, Absolute, CPU::and),
        op(0x2E, "ROL", 6, 3, false, Absolute, CPU::rol),
        nop(0x2F),

        // 0x30 – 0x3F
        op(0x30, "BMI", 2, 2, false, NoneAddressing, CPU::bmi),
        op(0x31, "AND", 5, 2, false, IndirectY, CPU::and),
        nop(0x32),
        nop(0x33),
        nop(0x34),
        op(0x35, "AND", 4, 2, false, ZeroPageX, CPU::and),
        op(0x36, "ROL", 6, 2, false, ZeroPageX, CPU::rol),
        nop(0x37),
        op(0x38, "SEC", 2, 1, false, NoneAddressing, CPU::sec),
        op(0x39, "AND", 4, 3, false, AbsoluteY, CPU::and),
        nop(0x3A),
        nop(0x3B),
        nop(0x3C),
        op(0x3D, "AND", 4, 3, false, AbsoluteX, CPU::and),
        op(0x3E, "ROL", 7, 3, false, AbsoluteX, CPU::rol),
        nop(0x3F),

        // 0x40 – 0x4F
        op(0x40, "RTI", 6, 1, false, NoneAddressing, CPU::rti),
        op(0x41, "EOR", 6, 2, false, IndirectX, CPU::eor),
        nop(0x42),
        nop(0x43),
        nop(0x44),
        op(0x45, "EOR", 3, 2, false, ZeroPage, CPU::eor),
        op(0x46, "LSR", 5, 2, false, ZeroPage, CPU::lsr),
        nop(0x47),
        op(0x48, "PHA", 3, 1, false, NoneAddressing, CPU::pha),
        op(0x49, "EOR", 2, 2, false, Immediate, CPU::eor),
        op(0x4A, "LSR", 2, 1, false, NoneAddressing, CPU::lsr),
        nop(0x4B),
        op(0x4C, "JMP", 3, 3, false, Absolute, CPU::jmp),
        op(0x4D, "EOR", 4, 3, false, Absolute, CPU::eor),
        op(0x4E, "LSR", 6, 3, false, Absolute, CPU::lsr),
        nop(0x4F),

        // 0x50 – 0x5F
        op(0x50, "BVC", 2, 2, false, NoneAddressing, CPU::bvc),
        op(0x51, "EOR", 5, 2, false, IndirectY, CPU::eor),
        nop(0x52),
        nop(0x53),
        nop(0x54),
        op(0x55, "EOR", 4, 2, false, ZeroPageX, CPU::eor),
        op(0x56, "LSR", 6, 2, false, ZeroPageX, CPU::lsr),
        nop(0x57),
        op(0x58, "CLI", 2, 1, false, NoneAddressing, CPU::cli),
        op(0x59, "EOR", 4, 3, false, AbsoluteY, CPU::eor),
        nop(0x5A),
        nop(0x5B),
        nop(0x5C),
        op(0x5D, "EOR", 4, 3, false, AbsoluteX, CPU::eor),
        op(0x5E, "LSR", 7, 3, false, AbsoluteX, CPU::lsr),
        nop(0x5F),

        // 0x60 – 0x6F
        op(0x60, "RTS", 6, 1, false, NoneAddressing, CPU::rts),
        op(0x61, "ADC", 6, 2, false, IndirectX, CPU::adc),
        nop(0x62),
        nop(0x63),
        nop(0x64),
        op(0x65, "ADC", 3, 2, false, ZeroPage, CPU::adc),
        op(0x66, "ROR", 5, 2, false, ZeroPage, CPU::ror),
        nop(0x67),
        op(0x68, "PLA", 4, 1, false, NoneAddressing, CPU::pla),
        op(0x69, "ADC", 2, 2, false, Immediate, CPU::adc),
        op(0x6A, "ROR", 2, 1, false, NoneAddressing, CPU::ror),
        nop(0x6B),
        op(0x6C, "JMP", 5, 3, false, NoneAddressing, CPU::jmp), // JMP (indirect)
        op(0x6D, "ADC", 4, 3, false, Absolute, CPU::adc),
        op(0x6E, "ROR", 6, 3, false, Absolute, CPU::ror),
        nop(0x6F),

        // 0x70 – 0x7F
        op(0x70, "BVS", 2, 2, false, NoneAddressing, CPU::bvs),
        op(0x71, "ADC", 5, 2, false, IndirectY, CPU::adc),
        nop(0x72),
        nop(0x73),
        nop(0x74),
        op(0x75, "ADC", 4, 2, false, ZeroPageX, CPU::adc),
        op(0x76, "ROR", 6, 2, false, ZeroPageX, CPU::ror),
        nop(0x77),
        op(0x78, "SEI", 2, 1, false, NoneAddressing, CPU::sei),
        op(0x79, "ADC", 4, 3, false, AbsoluteY, CPU::adc),
        nop(0x7A),
        nop(0x7B),
        nop(0x7C),
        op(0x7D, "ADC", 4, 3, false, AbsoluteX, CPU::adc),
        op(0x7E, "ROR", 7, 3, false, AbsoluteX, CPU::ror),
        nop(0x7F),

        // 0x80 – 0x8F
        nop(0x80),
        op(0x81, "STA", 6, 2, false, IndirectX, CPU::sta),
        nop(0x82),
        nop(0x83),
        op(0x84, "STY", 3, 2, false, ZeroPage, CPU::sty),
        op(0x85, "STA", 3, 2, false, ZeroPage, CPU::sta),
        op(0x86, "STX", 3, 2, false, ZeroPage, CPU::stx),
        nop(0x87),
        op(0x88, "DEY", 2, 1, false, NoneAddressing, CPU::dey),
        nop(0x89),
        op(0x8A, "TXA", 2, 1, false, NoneAddressing, CPU::txa),
        nop(0x8B),
        op(0x8C, "STY", 4, 3, false, Absolute, CPU::sty),
        op(0x8D, "STA", 4, 3, false, Absolute, CPU::sta),
        op(0x8E, "STX", 4, 3, false, Absolute, CPU::stx),
        nop(0x8F),

        // 0x90 – 0x9F
        op(0x90, "BCC", 2, 2, false, NoneAddressing, CPU::bcc),
        op(0x91, "STA", 6, 2, false, IndirectY, CPU::sta),
        nop(0x92),
        nop(0x93),
        op(0x94, "STY", 4, 2, false, ZeroPageX, CPU::sty),
        op(0x95, "STA", 4, 2, false, ZeroPageX, CPU::sta),
        op(0x96, "STX", 4, 2, false, ZeroPageY, CPU::stx),
        nop(0x97),
        op(0x98, "TYA", 2, 1, false, NoneAddressing, CPU::tya),
        op(0x99, "STA", 5, 3, false, AbsoluteY, CPU::sta),
        op(0x9A, "TXS", 2, 1, false, NoneAddressing, CPU::txs),
        nop(0x9B),
        nop(0x9C),
        op(0x9D, "STA", 5, 3, false, AbsoluteX, CPU::sta),
        nop(0x9E),
        nop(0x9F),

        // 0xA0 – 0xAF
        op(0xA0, "LDY", 2, 2, false, Immediate, CPU::ldy),
        op(0xA1, "LDA", 6, 2, false, IndirectX, CPU::lda),
        op(0xA2, "LDX", 2, 2, false, Immediate, CPU::ldx),
        nop(0xA3),
        op(0xA4, "LDY", 3, 2, false, ZeroPage, CPU::ldy),
        op(0xA5, "LDA", 3, 2, false, ZeroPage, CPU::lda),
        op(0xA6, "LDX", 3, 2, false, ZeroPage, CPU::ldx),
        nop(0xA7),
        op(0xA8, "TAY", 2, 1, false, NoneAddressing, CPU::tay),
        op(0xA9, "LDA", 2, 2, false, Immediate, CPU::lda),
        op(0xAA, "TAX", 2, 1, false, NoneAddressing, CPU::tax),
        nop(0xAB),
        op(0xAC, "LDY", 4, 3, false, Absolute, CPU::ldy),
        op(0xAD, "LDA", 4, 3, false, Absolute, CPU::lda),
        op(0xAE, "LDX", 4, 3, false, Absolute, CPU::ldx),
        nop(0xAF),

        // 0xB0 – 0xBF
        op(0xB0, "BCS", 2, 2, false, NoneAddressing, CPU::bcs),
        op(0xB1, "LDA", 5, 2, false, IndirectY, CPU::lda),
        nop(0xB2),
        nop(0xB3),
        op(0xB4, "LDY", 4, 2, false, ZeroPageX, CPU::ldy),
        op(0xB5, "LDA", 4, 2, false, ZeroPageX, CPU::lda),
        op(0xB6, "LDX", 4, 2, false, ZeroPageY, CPU::ldx),
        nop(0xB7),
        op(0xB8, "CLV", 2, 1, false, NoneAddressing, CPU::clv),
        op(0xB9, "LDA", 4, 3, false, AbsoluteY, CPU::lda),
        op(0xBA, "TSX", 2, 1, false, NoneAddressing, CPU::tsx),
        nop(0xBB),
        op(0xBC, "LDY", 4, 3, false, AbsoluteX, CPU::ldy),
        op(0xBD, "LDA", 4, 3, false, AbsoluteX, CPU::lda),
        op(0xBE, "LDX", 4, 3, false, AbsoluteY, CPU::ldx),
        nop(0xBF),

        // 0xC0 – 0xCF
        op(0xC0, "CPY", 2, 2, false, Immediate, CPU::cpy),
        op(0xC1, "CMP", 6, 2, false, IndirectX, CPU::cmp),
        nop(0xC2),
        nop(0xC3),
        op(0xC4, "CPY", 3, 2, false, ZeroPage, CPU::cpy),
        op(0xC5, "CMP", 3, 2, false, ZeroPage, CPU::cmp),
        op(0xC6, "DEC", 5, 2, false, ZeroPage, CPU::dec),
        nop(0xC7),
        op(0xC8, "INY", 2, 1, false, NoneAddressing, CPU::iny),
        op(0xC9, "CMP", 2, 2, false, Immediate, CPU::cmp),
        op(0xCA, "DEX", 2, 1, false, NoneAddressing, CPU::dex),
        nop(0xCB),
        op(0xCC, "CPY", 4, 3, false, Absolute, CPU::cpy),
        op(0xCD, "CMP", 4, 3, false, Absolute, CPU::cmp),
        op(0xCE, "DEC", 6, 3, false, Absolute, CPU::dec),
        nop(0xCF),

        // 0xD0 – 0xDF
        op(0xD0, "BNE", 2, 2, false, NoneAddressing, CPU::bne),
        op(0xD1, "CMP", 5, 2, false, IndirectY, CPU::cmp),
        nop(0xD2),
        nop(0xD3),
        nop(0xD4),
        op(0xD5, "CMP", 4, 2, false, ZeroPageX, CPU::cmp),
        op(0xD6, "DEC", 6, 2, false, ZeroPageX, CPU::dec),
        nop(0xD7),
        op(0xD8, "CLD", 2, 1, false, NoneAddressing, CPU::cld),
        op(0xD9, "CMP", 4, 3, false, AbsoluteY, CPU::cmp),
        nop(0xDA),
        nop(0xDB),
        nop(0xDC),
        op(0xDD, "CMP", 4, 3, false, AbsoluteX, CPU::cmp),
        op(0xDE, "DEC", 7, 3, false, AbsoluteX, CPU::dec),
        nop(0xDF),

        // 0xE0 – 0xEF
        op(0xE0, "CPX", 2, 2, false, Immediate, CPU::cpx),
        op(0xE1, "SBC", 6, 2, false, IndirectX, CPU::sbc),
        nop(0xE2),
        nop(0xE3),
        op(0xE4, "CPX", 3, 2, false, ZeroPage, CPU::cpx),
        op(0xE5, "SBC", 3, 2, false, ZeroPage, CPU::sbc),
        op(0xE6, "INC", 5, 2, false, ZeroPage, CPU::inc),
        nop(0xE7),
        op(0xE8, "INX", 2, 1, false, NoneAddressing, CPU::inx),
        op(0xE9, "SBC", 2, 2, false, Immediate, CPU::sbc),
        op(0xEA, "NOP", 2, 1, true, NoneAddressing, CPU::nop),
        nop(0xEB),
        op(0xEC, "CPX", 4, 3, false, Absolute, CPU::cpx),
        op(0xED, "SBC", 4, 3, false, Absolute, CPU::sbc),
        op(0xEE, "INC", 6, 3, false, Absolute, CPU::inc),
        nop(0xEF),

        // 0xF0 – 0xFF
        op(0xF0, "BEQ", 2, 2, false, NoneAddressing, CPU::beq),
        op(0xF1, "SBC", 5, 2, false, IndirectY, CPU::sbc),
        nop(0xF2),
        nop(0xF3),
        nop(0xF4),
        op(0xF5, "SBC", 4, 2, false, ZeroPageX, CPU::sbc),
        op(0xF6, "INC", 6, 2, false, ZeroPageX, CPU::inc),
        nop(0xF7),
        op(0xF8, "SED", 2, 1, false, NoneAddressing, CPU::sed),
        op(0xF9, "SBC", 4, 3, false, AbsoluteY, CPU::sbc),
        nop(0xFA),
        nop(0xFB),
        nop(0xFC),
        op(0xFD, "SBC", 4, 3, false, AbsoluteX, CPU::sbc),
        op(0xFE, "INC", 7, 3, false, AbsoluteX, CPU::inc),
        nop(0xFF),
    ];
    let opcode_map : HashMap<u8,OpCode> = instruction_vector.into_iter().map(|opcode| (opcode.opcode_hex,opcode)).collect();
    opcode_map



});

impl CPU {
    pub(crate) fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub(crate) fn mem_write(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }

    fn read_u16(&self, addr: u16) -> u16 {
        let lo = self.mem_read(addr) as u16;
        let hi = self.mem_read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    // --- flag bit helpers (operate directly on self.status) ---

    fn flag(&self, f: u8) -> bool {
        self.status & f != 0
    }

    fn set_flag(&mut self, f: u8, on: bool) {
        if on { self.status |= f; } else { self.status &= !f; }
    }

    fn update_zero_and_negative_flags(&mut self, value: u8) {
        self.set_flag(ZERO, value == 0);
        self.set_flag(NEGATIVE, value & 0x80 != 0);
    }

    // --- stack pointer in memory[0x00] ---

    fn sp(&self) -> u8 {
        self.mem_read(SP_ADDR)
    }

    fn set_sp(&mut self, value: u8) {
        self.mem_write(SP_ADDR, value);
    }

    fn push(&mut self, value: u8) {
        let addr = 0x0100 | self.sp() as u16;
        self.mem_write(addr, value);
        self.set_sp(self.sp().wrapping_sub(1));
    }

    fn pop(&mut self) -> u8 {
        self.set_sp(self.sp().wrapping_add(1));
        let addr = 0x0100 | self.sp() as u16;
        self.mem_read(addr)
    }

    fn push_u16(&mut self, value: u16) {
        self.push((value >> 8) as u8);
        self.push((value & 0xFF) as u8);
    }

    fn pop_u16(&mut self) -> u16 {
        let lo = self.pop() as u16;
        let hi = self.pop() as u16;
        (hi << 8) | lo
    }

    // --- addressing ---

    /// Advances PC past the operand, returns the effective address.
    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            Immediate => {
                let addr = self.program_counter;
                self.program_counter = self.program_counter.wrapping_add(1);
                addr
            }
            ZeroPage => {
                let addr = self.mem_read(self.program_counter) as u16;
                self.program_counter = self.program_counter.wrapping_add(1);
                addr
            }
            ZeroPageX => {
                let base = self.mem_read(self.program_counter);
                self.program_counter = self.program_counter.wrapping_add(1);
                base.wrapping_add(self.reg_x) as u16
            }
            ZeroPageY => {
                let base = self.mem_read(self.program_counter);
                self.program_counter = self.program_counter.wrapping_add(1);
                base.wrapping_add(self.reg_y) as u16
            }
            Absolute => {
                let addr = self.read_u16(self.program_counter);
                self.program_counter = self.program_counter.wrapping_add(2);
                addr
            }
            AbsoluteX => {
                let addr = self.read_u16(self.program_counter);
                self.program_counter = self.program_counter.wrapping_add(2);
                addr.wrapping_add(self.reg_x as u16)
            }
            AbsoluteY => {
                let addr = self.read_u16(self.program_counter);
                self.program_counter = self.program_counter.wrapping_add(2);
                addr.wrapping_add(self.reg_y as u16)
            }
            IndirectX => {
                let base = self.mem_read(self.program_counter);
                self.program_counter = self.program_counter.wrapping_add(1);
                let ptr = base.wrapping_add(self.reg_x);
                let lo = self.mem_read(ptr as u16) as u16;
                let hi = self.mem_read(ptr.wrapping_add(1) as u16) as u16;
                (hi << 8) | lo
            }
            IndirectY => {
                let base = self.mem_read(self.program_counter);
                self.program_counter = self.program_counter.wrapping_add(1);
                let lo = self.mem_read(base as u16) as u16;
                let hi = self.mem_read(base.wrapping_add(1) as u16) as u16;
                ((hi << 8) | lo).wrapping_add(self.reg_y as u16)
            }
            NoneAddressing => panic!("get_operand_address called with NoneAddressing"),
        }
    }

    fn branch(&mut self, condition: bool) {
        let offset = self.mem_read(self.program_counter) as i8;
        self.program_counter = self.program_counter.wrapping_add(1);
        if condition {
            self.program_counter = self.program_counter.wrapping_add_signed(offset as i16);
        }
    }

    fn compare(&mut self, register: u8, value: u8) {
        let result = register.wrapping_sub(value);
        self.set_flag(CARRY, register >= value);
        self.update_zero_and_negative_flags(result);
    }
}

// ============================================================
// Instructions
// ============================================================

impl CPU {
    // --- Load / Store ---
    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.reg_a = self.mem_read(addr);
        self.update_zero_and_negative_flags(self.reg_a);
    }
    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.reg_x = self.mem_read(addr);
        self.update_zero_and_negative_flags(self.reg_x);
    }
    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.reg_y = self.mem_read(addr);
        self.update_zero_and_negative_flags(self.reg_y);
    }
    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.reg_a);
    }
    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.reg_x);
    }
    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.reg_y);
    }

    // --- Transfers ---
    fn tax(&mut self, _m: &AddressingMode) { self.reg_x = self.reg_a; self.update_zero_and_negative_flags(self.reg_x); }
    fn tay(&mut self, _m: &AddressingMode) { self.reg_y = self.reg_a; self.update_zero_and_negative_flags(self.reg_y); }
    fn txa(&mut self, _m: &AddressingMode) { self.reg_a = self.reg_x; self.update_zero_and_negative_flags(self.reg_a); }
    fn tya(&mut self, _m: &AddressingMode) { self.reg_a = self.reg_y; self.update_zero_and_negative_flags(self.reg_a); }
    fn tsx(&mut self, _m: &AddressingMode) { self.reg_x = self.sp();  self.update_zero_and_negative_flags(self.reg_x); }
    fn txs(&mut self, _m: &AddressingMode) { self.set_sp(self.reg_x); } // no flags

    // --- Stack ---
    fn pha(&mut self, _m: &AddressingMode) { self.push(self.reg_a); }
    fn php(&mut self, _m: &AddressingMode) { self.push(self.status | BREAK | UNUSED); }
    fn pla(&mut self, _m: &AddressingMode) {
        self.reg_a = self.pop();
        self.update_zero_and_negative_flags(self.reg_a);
    }
    fn plp(&mut self, _m: &AddressingMode) {
        self.status = self.pop() | UNUSED;
    }

    // --- Arithmetic ---
    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let carry_in = self.flag(CARRY) as u16;
        let sum = self.reg_a as u16 + value as u16 + carry_in;
        self.set_flag(CARRY, sum > 0xFF);
        self.set_flag(OVERFLOW, (!(self.reg_a ^ value) & (self.reg_a ^ sum as u8) & 0x80) != 0);
        self.reg_a = sum as u8;
        self.update_zero_and_negative_flags(self.reg_a);
    }
    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let borrow_in = (!self.flag(CARRY)) as u8;
        let (a, b1) = self.reg_a.overflowing_sub(value);
        let (result, b2) = a.overflowing_sub(borrow_in);
        self.set_flag(CARRY, !(b1 || b2));
        self.set_flag(OVERFLOW, ((self.reg_a ^ value) & (self.reg_a ^ result) & 0x80) != 0);
        self.reg_a = result;
        self.update_zero_and_negative_flags(self.reg_a);
    }

    // --- Logical ---
    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.reg_a &= self.mem_read(addr);
        self.update_zero_and_negative_flags(self.reg_a);
    }
    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.reg_a ^= self.mem_read(addr);
        self.update_zero_and_negative_flags(self.reg_a);
    }
    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.reg_a |= self.mem_read(addr);
        self.update_zero_and_negative_flags(self.reg_a);
    }
    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_flag(ZERO, (self.reg_a & value) == 0);
        self.set_flag(NEGATIVE, value & 0x80 != 0);
        self.set_flag(OVERFLOW, value & 0x40 != 0);
    }

    // --- Shifts / Rotates (NoneAddressing = accumulator) ---
    fn asl(&mut self, mode: &AddressingMode) {
        if matches!(mode, NoneAddressing) {
            self.set_flag(CARRY, self.reg_a & 0x80 != 0);
            self.reg_a <<= 1;
            self.update_zero_and_negative_flags(self.reg_a);
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            self.set_flag(CARRY, value & 0x80 != 0);
            let result = value << 1;
            self.mem_write(addr, result);
            self.update_zero_and_negative_flags(result);
        }
    }
    fn lsr(&mut self, mode: &AddressingMode) {
        if matches!(mode, NoneAddressing) {
            self.set_flag(CARRY, self.reg_a & 0x01 != 0);
            self.reg_a >>= 1;
            self.update_zero_and_negative_flags(self.reg_a);
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            self.set_flag(CARRY, value & 0x01 != 0);
            let result = value >> 1;
            self.mem_write(addr, result);
            self.update_zero_and_negative_flags(result);
        }
    }
    fn rol(&mut self, mode: &AddressingMode) {
        if matches!(mode, NoneAddressing) {
            let carry_in = self.flag(CARRY) as u8;
            self.set_flag(CARRY, self.reg_a & 0x80 != 0);
            self.reg_a = (self.reg_a << 1) | carry_in;
            self.update_zero_and_negative_flags(self.reg_a);
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let carry_in = self.flag(CARRY) as u8;
            self.set_flag(CARRY, value & 0x80 != 0);
            let result = (value << 1) | carry_in;
            self.mem_write(addr, result);
            self.update_zero_and_negative_flags(result);
        }
    }
    fn ror(&mut self, mode: &AddressingMode) {
        if matches!(mode, NoneAddressing) {
            let carry_in = (self.flag(CARRY) as u8) << 7;
            self.set_flag(CARRY, self.reg_a & 0x01 != 0);
            self.reg_a = (self.reg_a >> 1) | carry_in;
            self.update_zero_and_negative_flags(self.reg_a);
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let carry_in = (self.flag(CARRY) as u8) << 7;
            self.set_flag(CARRY, value & 0x01 != 0);
            let result = (value >> 1) | carry_in;
            self.mem_write(addr, result);
            self.update_zero_and_negative_flags(result);
        }
    }

    // --- Jumps / Calls / Interrupts ---
    fn jmp(&mut self, mode: &AddressingMode) {
        match mode {
            Absolute => {
                self.program_counter = self.read_u16(self.program_counter);
            }
            // JMP (indirect) — table maps it to NoneAddressing
            NoneAddressing => {
                let ptr = self.read_u16(self.program_counter);
                let lo = self.mem_read(ptr) as u16;
                // page-bug: high byte wraps inside the pointer's page
                let hi_addr = (ptr & 0xFF00) | ((ptr as u8).wrapping_add(1) as u16);
                let hi = self.mem_read(hi_addr) as u16;
                self.program_counter = (hi << 8) | lo;
            }
            _ => unreachable!("JMP: Absolute or Indirect only"),
        }
    }
    fn jsr(&mut self, mode: &AddressingMode) {
        let target = self.get_operand_address(mode);
        let return_addr = self.program_counter.wrapping_sub(1);
        self.push_u16(return_addr);
        self.program_counter = target;
    }
    fn rts(&mut self, _m: &AddressingMode) {
        self.program_counter = self.pop_u16().wrapping_add(1);
    }
    fn rti(&mut self, _m: &AddressingMode) {
        self.status = self.pop() | UNUSED;
        self.program_counter = self.pop_u16();
    }
    fn brk(&mut self, _m: &AddressingMode) {
        let return_addr = self.program_counter.wrapping_add(1);
        self.push_u16(return_addr);
        self.push(self.status | BREAK | UNUSED);
        self.set_flag(INTERRUPT_DISABLE, true);
        let lo = self.mem_read(0xFFFE) as u16;
        let hi = self.mem_read(0xFFFF) as u16;
        self.program_counter = (hi << 8) | lo;
    }

    // --- Branches ---
    fn bpl(&mut self, _m: &AddressingMode) { self.branch(!self.flag(NEGATIVE)); }
    fn bmi(&mut self, _m: &AddressingMode) { self.branch(self.flag(NEGATIVE)); }
    fn bvc(&mut self, _m: &AddressingMode) { self.branch(!self.flag(OVERFLOW)); }
    fn bvs(&mut self, _m: &AddressingMode) { self.branch(self.flag(OVERFLOW)); }
    fn bcc(&mut self, _m: &AddressingMode) { self.branch(!self.flag(CARRY)); }
    fn bcs(&mut self, _m: &AddressingMode) { self.branch(self.flag(CARRY)); }
    fn bne(&mut self, _m: &AddressingMode) { self.branch(!self.flag(ZERO)); }
    fn beq(&mut self, _m: &AddressingMode) { self.branch(self.flag(ZERO)); }

    // --- Compare ---
    fn cmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.compare(self.reg_a, value);
    }
    fn cpx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.compare(self.reg_x, value);
    }
    fn cpy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.compare(self.reg_y, value);
    }

    // --- Inc / Dec ---
    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr).wrapping_add(1);
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }
    fn inx(&mut self, _m: &AddressingMode) {
        self.reg_x = self.reg_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.reg_x);
    }
    fn iny(&mut self, _m: &AddressingMode) {
        self.reg_y = self.reg_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.reg_y);
    }
    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr).wrapping_sub(1);
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }
    fn dex(&mut self, _m: &AddressingMode) {
        self.reg_x = self.reg_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.reg_x);
    }
    fn dey(&mut self, _m: &AddressingMode) {
        self.reg_y = self.reg_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.reg_y);
    }

    // --- Flags ---
    fn clc(&mut self, _m: &AddressingMode) { self.status &= !CARRY; }
    fn cld(&mut self, _m: &AddressingMode) { self.status &= !DECIMAL; }
    fn cli(&mut self, _m: &AddressingMode) { self.status &= !INTERRUPT_DISABLE; }
    fn clv(&mut self, _m: &AddressingMode) { self.status &= !OVERFLOW; }
    fn sec(&mut self, _m: &AddressingMode) { self.status |=  CARRY; }
    fn sed(&mut self, _m: &AddressingMode) { self.status |=  DECIMAL; }
    fn sei(&mut self, _m: &AddressingMode) { self.status |=  INTERRUPT_DISABLE; }

    // --- System ---
    fn nop(&mut self, _m: &AddressingMode) {}
}

// ============================================================
// Dispatch
// ===
