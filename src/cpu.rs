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
}

pub struct Instruction {
    opcode: u8,
    size: u8,
    disassembly: String,
    func_symbols: String,
    effected_flags: String,
    func_ptr: fn(&mut Cpu8080, u8, u8)
}

pub struct Cpu8080 {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8, c: u8,
    d: u8, e: u8,
    h: u8, l: u8,

    memory: Vec<u8>,
    condition_codes: ConditionBitset,

    opcode_table: std::collections::HashMap<u8, Instruction>
}

impl Cpu8080 {

    pub fn execute(&mut self, opcode: u8) {
        self.pc += 1;

        if let Some(i) = self.opcode_table.get(&opcode) {
            let func = i.func_ptr;
            let pc = self.pc as usize;
            match i.size {
                1 => { func(self, 0, 0) },
                2 => { func(self, self.memory[pc], 0); self.pc += 1; },
                3 => { func(self, self.memory[pc], self.memory[pc + 1]); self.pc += 2; },
                _ => {}
            }
        } else {
            // TODO :: Do something lol
            panic!("OPCODE ERROR :: Opcode {:x} not found", opcode);
        }

    }


}

fn fill_opcode_table(mut optable: HashMap<u8, Instruction>) {
    optable.insert(0x00, Instruction { opcode: 0x00, size: 1, disassembly: "NOP".into(),       func_symbols: "".into(),                         effected_flags: String::new(), func_ptr: |_, _, _| { } });
    optable.insert(0x01, Instruction { opcode: 0x01, size: 3, disassembly: "LXI B,D16".into(), func_symbols: "B <- byte 3, C <- byte 2".into(), effected_flags: String::new(), func_ptr: |cpu, b2, b3| { cpu.b = b3; cpu.c = b2; } });


    
}


pub fn read_file(path: &str) -> std::io::Result<Vec<u8>> {
    use std::fs::File;
    use std::io::prelude::*;

    let mut f = File::open(path)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    Ok(buffer)
}
