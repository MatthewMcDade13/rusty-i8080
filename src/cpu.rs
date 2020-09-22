pub mod opcode;
use opcode::{Instruction, OpcodeTable};
use std::num::Wrapping;
use std::collections::HashMap;


#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum ConditionFlag {
    Zero = 1 << 0, 
    Sign = 1 << 1, 
    Parity = 1 << 2, 
    Carry = 1 << 3, 
    AuxiliaryCarry = 1 << 4
}

pub struct ConditionBitset(u8);

type HighU8 = u8;
type LowU8 = u8;

impl ConditionBitset {
    pub fn set(&mut self, flag: ConditionFlag) {
        self.0 &= flag as u8;
    }

    pub fn unset(&mut self, flag: ConditionFlag) {
        self.0 &= !(flag as u8)
    }

    pub fn is_set(&self, flag: ConditionFlag) -> bool {
        (self.0 & flag as u8) == flag as u8
    }

    fn as_bit(&self, flag: ConditionFlag) -> u8 {
        if self.is_set(flag) { 1 } else { 0 }
    }
}

pub struct Cpu8080 {
    pub pc: u16,
    pub sp: u16,
    pub a: u8,
    pub b: u8, pub c: u8,
    pub d: u8, pub e: u8,
    pub h: u8, pub l: u8,

    pub halted: bool,

    pub memory: Vec<u8>,
    pub condition_codes: ConditionBitset,

    pub opcode_table: OpcodeTable
}

impl Cpu8080 {

    // pub fn new() -> Self {
    //     Cpu8080 {
    //         pc
    //     }
    // }

    pub fn execute(&mut self, opcode: u8) {
        self.pc += 1;

        if let Some(instruction) = self.opcode_table.get(opcode) {
            let op = instruction.func_ptr;
            let pc = self.pc as usize;
            match instruction.size {
                1 => { op(self, 0, 0) },
                2 => { op(self, self.memory[pc], 0); self.pc += 1; },
                3 => { op(self, self.memory[pc], self.memory[pc + 1]); self.pc += 2; },
                _ => {}
            }
        } else {
            // TODO :: Do something lol
            panic!("OPCODE ERROR :: Opcode {:x} not found", opcode);
        }

    }

    /**
     * Increments stack pointer by 2 then pops value off of stack, which is returned in its parts: (high, low)
     * */
    pub fn pop_stack_parts(&mut self) -> (HighU8, LowU8) {
        self.sp += 2;
        self.read_u16_parts(self.sp)
    }

    pub fn pop_stack(&mut self) -> u16 {
        let (high, low) = self.pop_stack_parts();
        combine_bytes(high, low)
    }

    pub fn push_stack(&mut self, val: u16) {
        self.write_u16(self.sp - 2, val);
        self.sp -= 2;
    }

    /**
     * Reads u16 from memory address and returns it in its parts: (high, low)
     * */
    pub fn read_u16_parts(&self, addr: u16) -> (HighU8, LowU8) {
        let addr = addr as usize;
        let low = self.memory[addr];
        let high = self.memory[addr + 1];
        (high, low)
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let (high, low) = self.read_u16_parts(addr);
        combine_bytes(high, low)
    }

    pub fn write_u16(&mut self, addr: u16, val: u16) {
        let addr = addr as usize;

        let mut high = self.memory[addr + 1];
        let mut low = self.memory[addr];
        set_byte_pair(&mut high, &mut low, val);
        self.memory[addr + 1] = high;
        self.memory[addr] = low;
    }

    pub fn jmp(&mut self, addr: u16) {
        self.pc = addr;
    }

    pub fn ret(&mut self) {
        self.pc = self.pop_stack();
        // self.pc = self.read_u16(self.sp);
        // self.sp += 2;
    }

    pub fn call(&mut self, addr: u16) {
        self.push_stack(self.pc);
        self.pc = addr;
    }

    pub fn adc(&mut self, val: u8) {
        use Wrapping as W;
        let (a, val, cy) = (self.a as u16, val as u16, self.condition_codes.as_bit(ConditionFlag::Carry) as u16);

        let result = W(a) + W(val) + W(cy);
        let result = result.0;

        self.check_zero(result);
        self.check_sign(result);
        self.check_parity(result as u32);
        self.check_carry(result);
        // aux carry
        self.a = result as u8;
    }

    pub fn sbb(&mut self, val: u8) {
        use Wrapping as W;
        let (a, val, cy)  = (self.a as u16, val as u16, self.condition_codes.as_bit(ConditionFlag::Carry) as u16);

        let result = W(a) - W(val) - W(cy);
        let result = result.0;

        self.check_zero(result);
        self.check_sign(result);
        self.check_parity(result as u32);
        self.check_carry(result);
        // aux carry
        self.a = result as u8;
    }

    pub fn add(&mut self, val: u8) {
        let result = Wrapping(self.a as u16) + Wrapping(val as u16);
        let result = result.0;

        self.check_zero(result);
        self.check_sign(result);
        self.check_parity(result as u32);
        self.check_carry(result);
        // aux carry

        self.a = result as u8;
    }

    pub fn sub(&mut self, val: u8) {
        let result = Wrapping(self.a as u16) - Wrapping(val as u16);
        let result = result.0;

        self.check_zero(result);
        self.check_sign(result);
        self.check_parity(result as u32);
        self.check_carry(result);
        // aux carry

        self.a = result as u8;
    }

    pub fn inr(&mut self, val: u8) -> u8 {
        let result = Wrapping(val as u16) + Wrapping(1);
        let result = result.0;

        self.check_zero(result);
        self.check_sign(result);
        self.check_parity(result as u32);
        // auxillary carry

        result as u8
    }

    pub fn dcr(&mut self, val: u8) -> u8 {
        let result = Wrapping(val as u16) - Wrapping(1);
        let result = result.0;

        self.check_zero(result);
        self.check_sign(result);
        self.check_parity(result as u32);
        // auxillary carry

        result as u8
    }

    pub fn ana(&mut self, val: u8) {
        let result = (self.a as u16) & (val as u16);

        self.check_carry(result);
        self.check_zero(result);
        self.check_sign(result);
        self.check_parity(result as u32);
        // aux carry

        self.a = result as u8;
    }

    pub fn xra(&mut self, val: u8) {
        let result = (self.a as u16) ^ (val as u16);

        self.check_carry(result);
        self.check_zero(result);
        self.check_sign(result);
        self.check_parity(result as u32);
        // aux carry

        self.a = result as u8;
    }

    pub fn ora(&mut self, val: u8) {
        let result = (self.a as u16) | (val as u16);

        self.check_carry(result);
        self.check_zero(result);
        self.check_sign(result);
        self.check_parity(result as u32);
        // aux carry

        self.a = result as u8;
    }

    pub fn cmp(&mut self, val: u8) {
        let result = Wrapping(self.a as u16) - Wrapping(val as u16);
        let result = result.0;

        self.check_zero(result);
        self.check_carry(result);
        self.check_sign(result);
        self.check_parity(result as u32);
    }

    fn check_zero(&mut self, result: u16) -> bool {
        if result & 0xFF == 0 {
            self.condition_codes.set(ConditionFlag::Zero);
            true
        } else {
            self.condition_codes.unset(ConditionFlag::Zero);
            false
        }
    }

    fn check_sign(&mut self, result: u16) -> bool {
        if result & 0x80 != 0 {
            self.condition_codes.set(ConditionFlag::Sign);
            true
        } else {
            self.condition_codes.unset(ConditionFlag::Sign);
            false
        }
    }

    fn check_parity(&mut self, result: u32) -> bool {
        if parity(result) {
            self.condition_codes.set(ConditionFlag::Parity);
            true
        } else {
            self.condition_codes.unset(ConditionFlag::Parity);
            false
        }
    }

    fn check_carry(&mut self, result: u16) -> bool {
        if result > 0xFF {
            self.condition_codes.set(ConditionFlag::Carry);
            true
        } else {
            self.condition_codes.unset(ConditionFlag::Carry);
            false
        }
    }
}

pub(crate) fn set_byte_pair(high: &mut u8, low: &mut u8, scalar: u16) {
    *low = scalar as u8;
    *high = (scalar >> 8) as u8;
}


// little endian
pub(crate) const fn combine_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | low as u16
}

// from https://www.tutorialspoint.com/cplusplus-program-to-find-the-parity-of-a-number-efficiently
pub(crate) const fn parity(n: u32) -> bool {
    let mut y = n ^ (n >> 1);
    y = y ^ (y >> 2);
    y = y ^ (y >> 4);
    y = y ^ (y >> 6);
    y = y ^ (y >> 8);
    y = y ^ (y >> 16);
    (y & 1) != 0
}

pub(crate) const fn sign_flag(n: u16) -> bool {
    n & 0x80 != 0
}

pub(crate) const fn carry_flag(n: u16) -> bool {
    n > 0xFF
}

pub fn read_file(path: &str) -> std::io::Result<Vec<u8>> {
    use std::fs::File;
    use std::io::prelude::*;

    let mut f = File::open(path)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    Ok(buffer)
}
