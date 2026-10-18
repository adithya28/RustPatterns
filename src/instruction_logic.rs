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
use crate::instruction_spec::AddressingMode;
// ============================================================
// Primitives
// ============================================================

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
