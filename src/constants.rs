pub const CARRY: u8             = 0b0000_0001; // C  - bit 0
pub const ZERO: u8              = 0b0000_0010; // Z  - bit 1
pub const INTERRUPT_DISABLE: u8 = 0b0000_0100; // I  - bit 2
pub const DECIMAL: u8           = 0b0000_1000; // D  - bit 3
pub const BREAK: u8             = 0b0001_0000; // B  - bit 4
pub const UNUSED: u8            = 0b0010_0000; // U  - bit 5 (always 1 on real chip)
pub const OVERFLOW: u8          = 0b0100_0000; // V  - bit 6
pub const NEGATIVE: u8          = 0b1000_0000; // N  - bit 7

pub const SP_ADDR: u16 = 0x0000;
