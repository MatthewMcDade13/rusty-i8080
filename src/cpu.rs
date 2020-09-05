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
    disassembly: &'static str,
    func_symbols: &'static str,
    effected_flags: Option<&'static str>,
    func_ptr: fn(&mut Cpu8080, b2: u8, b3: u8)
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

    fn inr(&mut self, val: u8) -> u8 {
        let result = val as u16 + 1;

        self.check_zero(result);
        self.check_sign(result);
        self.check_parity(result as u32);
        // auxillary carry

        result as u8
    }

    fn dcr(&mut self, val: u8) -> u8 {
        let result = val as u16 - 1;

        self.check_zero(result);
        self.check_sign(result);
        self.check_parity(result as u32);
        // auxillary carry

        result as u8
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

fn fill_opcode_table(optable: &mut HashMap<u8, Instruction>) {

    insert_instruction(optable, Instruction { opcode: 0x00, size: 1, disassembly: "NOP",       func_symbols: "",                         effected_flags: None, 
        func_ptr: |_, _, _| { } 
    });
    insert_instruction(optable, Instruction { opcode: 0x01, size: 3, disassembly: "LXI B,D16", func_symbols: "B <- byte 3, C <- byte 2", effected_flags: None, 
        func_ptr: |cpu, b2, b3| { cpu.b = b3; cpu.c = b2; } 
    });
    insert_instruction(optable, Instruction { opcode: 0x02, size: 1, disassembly: "STAX B",    func_symbols: "(BC) <- A",                effected_flags: None, 
        func_ptr: |cpu, _, _| { cpu.memory[combine_bytes(cpu.b, cpu.c) as usize] = cpu.a } 
    });

    insert_instruction(optable, Instruction { opcode: 0x03, size: 1, disassembly: "INX B",     func_symbols: "BC <- BC + 1",             effected_flags: None, 
        func_ptr: |cpu, _, _|  { 
            let r = combine_bytes(cpu.b, cpu.c) + 1; 
            set_register_pair(&mut cpu.b, &mut cpu.c, r) 
        } 
    });
    insert_instruction(optable, Instruction { opcode: 0x04, size: 1, disassembly: "INR B",     func_symbols: "B <- B + 1", effected_flags: "Z,S,P,AC".into(), 
        func_ptr: |cpu, _, _|   { 
            cpu.b = cpu.inr(cpu.b);
        } 
    });

    insert_instruction(optable, Instruction { opcode: 0x05, size: 1, disassembly: "DCR B", func_symbols: "B <- B - 1", effected_flags: "Z,S,P,AC".into(), 
        func_ptr: |cpu, _, _| { 
            cpu.b = cpu.dcr(cpu.b)
        } 
    });

    insert_instruction(optable, Instruction { opcode: 0x06, size: 2, disassembly: "MVI B, D8", func_symbols: "B <- byte 2", effected_flags: None,
        func_ptr: |cpu, b2, _| { 
            cpu.b = b2;
        } 
    });

    insert_instruction(optable, Instruction { opcode: 0x07, size: 1, disassembly: "RLC", func_symbols: "A = A << 1; bit 0 = prev bit 7; CY = prev bit 7", effected_flags: "CY".into(),
        func_ptr: |cpu, _, _| { 
            let rotated = cpu.a >> 7;
            cpu.a = (cpu.a << 1) | rotated;
            
            if rotated != 0 { 
                cpu.condition_codes.set(ConditionFlag::Carry) 
            } else {
                cpu.condition_codes.unset(ConditionFlag::Carry)
            }
        } 
    });

    insert_instruction(optable, Instruction { opcode: 0x09, size: 1, disassembly: "DAD B", func_symbols: "HL = HL + BC", effected_flags: "CY".into(),
        func_ptr: |cpu, _, _| { 
            let hl = combine_bytes(cpu.h, cpu.l);
            let bc = combine_bytes(cpu.b, cpu.c);
            let result = hl + bc;
            cpu.check_carry(result);

            set_register_pair(&mut cpu.h, &mut cpu.l, result);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x0A, size: 1, disassembly: "LDAX B", func_symbols: "A <- (BC)", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let bc = combine_bytes(cpu.b, cpu.c) as usize;
            cpu.a = cpu.memory[bc];
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x0B, size: 1, disassembly: "DCX B", func_symbols: "BC = BC-1", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let bc = combine_bytes(cpu.b, cpu.c);
            set_register_pair(&mut cpu.b, &mut cpu.c, bc - 1);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x0C, size: 1, disassembly: "INR C", func_symbols: "C <- C + 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            cpu.c = cpu.inr(cpu.c);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x0D, size: 1, disassembly: "DCR C", func_symbols: "C <- C - 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            cpu.c = cpu.dcr(cpu.c);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x0E, size: 2, disassembly: "MVI C, D8", func_symbols: "C <- byte 2", effected_flags: None,
        func_ptr: |cpu, b2, _| { 
            cpu.c = b2;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x0F, size: 1, disassembly: "RRC", func_symbols: "A = A >> 1; bit 7 = prev bit 0; CY = prev bit 0", effected_flags: "CY".into(),
        func_ptr: |cpu, _, _| { 
            let rotated = cpu.a << 7;
            cpu.a = (cpu.a >> 1) | rotated;
            
            if rotated != 0 { 
                cpu.condition_codes.set(ConditionFlag::Carry) 
            } else {
                cpu.condition_codes.unset(ConditionFlag::Carry)
            }
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x11, size: 3, disassembly: "LXI D,D16", func_symbols: "D <- byte 3, E <- byte 2", effected_flags: None,
        func_ptr: |cpu, b2, b3| { 
            cpu.d = b3;
            cpu.e = b2;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x12, size: 1, disassembly: "STAX D", func_symbols: "(DE) <- A", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let de = combine_bytes(cpu.d, cpu.e) as usize;
            cpu.memory[de] = cpu.a;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x13, size: 1, disassembly: "INX D", func_symbols: "DE <- DE + 1", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let result = combine_bytes(cpu.d, cpu.e) + 1;
            set_register_pair(&mut cpu.d, &mut cpu.e, result);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x14, size: 1, disassembly: "INR D", func_symbols: "D <- D + 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            cpu.d = cpu.inr(cpu.d);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x15, size: 1, disassembly: "DCR D", func_symbols: "D <- D - 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            cpu.d = cpu.dcr(cpu.d);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x16, size: 2, disassembly: "MVI D, D8", func_symbols: "D <- byte 2", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let result = cpu.d as u16 - 1;
            cpu.check_zero(result);
            cpu.check_sign(result);
            cpu.check_parity(result as u32);
            // aux carry
            cpu.d = result as u8;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x17, size: 1, disassembly: "RAL", func_symbols: "A = A << 1; bit 0 = prev CY; CY = prev bit 7", effected_flags: "CY".into(),
        func_ptr: |cpu, _, _| { 
            let prev_carry = if cpu.condition_codes.is_set(ConditionFlag::Carry) { 1 } else { 0 };
            let rotated = cpu.a >> 7;
            cpu.a = (cpu.a << 1) | prev_carry;
            
            if rotated != 0 { 
                cpu.condition_codes.set(ConditionFlag::Carry) 
            } else {
                cpu.condition_codes.unset(ConditionFlag::Carry)
            }
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x19, size: 1, disassembly: "DAD D", func_symbols: "HL = HL + DE", effected_flags: "CY".into(),
        func_ptr: |cpu, _, _| { 
            let hl = combine_bytes(cpu.h, cpu.l);
            let de = combine_bytes(cpu.d, cpu.e);
            let result = hl + de;

            cpu.check_carry(result);

            set_register_pair(&mut cpu.h, &mut cpu.l, result);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x1A, size: 1, disassembly: "LDAX D", func_symbols: "A <- (DE)", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let de = combine_bytes(cpu.d, cpu.e) as usize;
            cpu.a = cpu.memory[de];
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x1B, size: 1, disassembly: "DCX D", func_symbols: "DE <- DE - 1", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let result = combine_bytes(cpu.d, cpu.e) - 1;
            set_register_pair(&mut cpu.d, &mut cpu.e, result);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x1C, size: 1, disassembly: "INR E", func_symbols: "E <- E + 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            cpu.e = cpu.inr(cpu.e);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x1D, size: 1, disassembly: "DCR E", func_symbols: "E <- E - 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            cpu.e = cpu.dcr(cpu.e);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x1E, size: 2, disassembly: "MVI E, D8", func_symbols: "E <- byte 2", effected_flags: None,
        func_ptr: |cpu, b2, _| { 
            cpu.e = b2;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x1F, size: 1, disassembly: "RAR", func_symbols: "A = A >> 1; bit 7 = prev CY; CY = prev bit 0", effected_flags: "CY".into(),
        func_ptr: |cpu, _, _| { 
            let prev_carry = if cpu.condition_codes.is_set(ConditionFlag::Carry) { 1 } else { 0 };
            let rotated = cpu.a << 7;
            cpu.a = (cpu.a >> 1) | (prev_carry << 7);
            
            if rotated != 0 { 
                cpu.condition_codes.set(ConditionFlag::Carry) 
            } else {
                cpu.condition_codes.unset(ConditionFlag::Carry)
            }
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x21, size: 3, disassembly: "LXI H, D16", func_symbols: "H <- byte 3, L <- byte 2", effected_flags: None,
        func_ptr: |cpu, b2, b3| { 
            cpu.h = b3;
            cpu.l = b2;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x22, size: 3, disassembly: "SHLD adr", func_symbols: "(adr) <- L; (adr + 1) <- H", effected_flags: None,
        func_ptr: |cpu, b2, b3| { 
            let addr = combine_bytes(b3, b2) as usize;
            cpu.memory[addr] = cpu.l;
            cpu.memory[addr + 1] = cpu.h;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x23, size: 1, disassembly: "INX H", func_symbols: "HL <- HL + 1", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let result = combine_bytes(cpu.h, cpu.l);
            set_register_pair(&mut cpu.h, &mut cpu.l, result + 1);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x24, size: 1, disassembly: "INR H", func_symbols: "H <- H + 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            cpu.h = cpu.inr(cpu.h);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x25, size: 1, disassembly: "DCR H", func_symbols: "H <- H - 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            cpu.h = cpu.dcr(cpu.h);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x26, size: 2, disassembly: "MVI H, D8", func_symbols: "H <- byte 2", effected_flags: None,
        func_ptr: |cpu, b2, _| { 
            cpu.h = b2;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x27, size: 1, disassembly: "DAA", func_symbols: "Decimal Adjust Accumulator", effected_flags: "Z,S,P,CY,AC".into(),
        func_ptr: |cpu, _, _| { 
            /*
             * The 8 bit number in the accumulator is adjusted to form 2 four-bit Binary-Coded-Decimal digits by the following process
             * 
             *  1. If the value of the least significant 4 bits of the accumulator is greater than 9 OR if the AC flag is set, 6 is added to the accumulator
             *  2. If the value of the most significant 4 bits of the accumulator is now greater than 9 OR if the CY flag is set, 6 is added to the most 4 significant bits of the accumulator
             */
            let lsb = cpu.a & 0xF;
            let msb = cpu.a & (0xF << 4);

            let mut acc = 0;
            
            if lsb > 9 || cpu.condition_codes.is_set(ConditionFlag::AuxiliaryCarry) {
                acc += 0x06;
            }
            
            if msb > 9 || cpu.condition_codes.is_set(ConditionFlag::Carry) {
                acc += 0x60;
            }
            let result = cpu.a as u16 + acc;
            cpu.check_zero(result);
            cpu.check_sign(result);
            cpu.check_parity(result as u32);
            cpu.check_carry(result);
            // aux carry

            cpu.a = result as u8;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x29, size: 1, disassembly: "DAD H", func_symbols: "HL <- HL + HL", effected_flags: "CY".into(),
        func_ptr: |cpu, _, _| { 
            let hl = combine_bytes(cpu.h, cpu.l) as u16;
            let result = hl + hl;
            cpu.check_carry(result);
            set_register_pair(&mut cpu.h, &mut cpu.l, result);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x2A, size: 3, disassembly: "LHLD adr", func_symbols: "L <- (adr); H <- (adr + 1)", effected_flags: None,
        func_ptr: |cpu, b2, b3| { 
            let addr = combine_bytes(b3, b2) as usize;
            cpu.l = cpu.memory[addr];
            cpu.h = cpu.memory[addr + 1];
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x2B, size: 1, disassembly: "DCX H", func_symbols: "HL <- HL - 1", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let hl = combine_bytes(cpu.h, cpu.l);
            set_register_pair(&mut cpu.h, &mut cpu.l, hl + 1);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x2C, size: 1, disassembly: "INR L", func_symbols: "L <- L + 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            cpu.l = cpu.inr(cpu.l);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x2D, size: 1, disassembly: "DCR L", func_symbols: "L <- L - 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            cpu.l = cpu.dcr(cpu.l);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x2E, size: 2, disassembly: "MVI L, D8", func_symbols: "L <- byte 2", effected_flags: None,
        func_ptr: |cpu, b2, _| { 
            cpu.l = b2;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x2F, size: 1, disassembly: "CMA", func_symbols: "A <- !A", effected_flags: None,
        func_ptr: |cpu, b2, _| { 
            cpu.a = !cpu.a;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x31, size: 3, disassembly: "LXI SP, D16", func_symbols: "SP.high <- byte 3; SP.low <- byte 2", effected_flags: None,
        func_ptr: |cpu, b2, b3| { 
            cpu.sp = combine_bytes(b3, b2); 
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x32, size: 3, disassembly: "STA adr", func_symbols: "(adr) <- A", effected_flags: None,
        func_ptr: |cpu, b2, b3| { 
            let addr = combine_bytes(b3, b2) as usize;
            cpu.memory[addr] = cpu.a;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x33, size: 1, disassembly: "INX SP", func_symbols: "SP = SP + 1", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.sp += 1;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x34, size: 1, disassembly: "INR M", func_symbols: "(HL) <- (HL) + 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            let hl = combine_bytes(cpu.h, cpu.l) as usize;
            cpu.memory[hl] = cpu.inr(cpu.memory[hl]);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x35, size: 1, disassembly: "DCR M", func_symbols: "(HL) <- (HL) - 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            let hl = combine_bytes(cpu.h, cpu.l) as usize;
            cpu.memory[hl] = cpu.dcr(cpu.memory[hl]);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x36, size: 2, disassembly: "MVI M, D8", func_symbols: "(HL) <- byte 2", effected_flags: None,
        func_ptr: |cpu, b2, _| { 
            let hl = combine_bytes(cpu.h, cpu.l) as usize;
            cpu.memory[hl] = b2;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x37, size: 1, disassembly: "STC", func_symbols: "CY = 1", effected_flags: "CY".into(),
        func_ptr: |cpu, _, _| { 
            cpu.condition_codes.set(ConditionFlag::Carry);                
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x39, size: 1, disassembly: "DAD SP", func_symbols: "HL <- HL + SP", effected_flags: "CY".into(),
        func_ptr: |cpu, _, _| { 
            let hl = combine_bytes(cpu.h, cpu.l);
            let result = hl + cpu.sp;
            cpu.check_carry(result);
            set_register_pair(&mut cpu.h, &mut cpu.l, result);           
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x3A, size: 3, disassembly: "LDA adr", func_symbols: "A <- (adr)", effected_flags: None,
        func_ptr: |cpu, b2, b3| { 
            let addr = combine_bytes(b3, b2) as usize;
            cpu.a = cpu.memory[addr];
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x3B, size: 1, disassembly: "DCX SP", func_symbols: "SP <- SP + 1", effected_flags: None,
        func_ptr: |cpu, b2, b3| { 
            cpu.sp += 1;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x3C, size: 1, disassembly: "INR A", func_symbols: "A <- A + 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            cpu.a = cpu.inr(cpu.a);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x3D, size: 1, disassembly: "DCR A", func_symbols: "A <- A - 1", effected_flags: "Z,S,P,AC".into(),
        func_ptr: |cpu, _, _| { 
            cpu.a = cpu.dcr(cpu.a);
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x3E, size: 2, disassembly: "MVI A, D8", func_symbols: "A <- byte 2", effected_flags: None,
        func_ptr: |cpu, b2, _| { 
            cpu.a = b2;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x3F, size: 1, disassembly: "CMC", func_symbols: "CY = !CY", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            if cpu.condition_codes.is_set(ConditionFlag::Carry) {
                cpu.condition_codes.unset(ConditionFlag::Carry);
            } else {
                cpu.condition_codes.set(ConditionFlag::Carry)
            }
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x40, size: 1, disassembly: "MOV B, B", func_symbols: "B <- B", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.b = cpu.b;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x40, size: 1, disassembly: "MOV B, B", func_symbols: "B <- B", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.b = cpu.b;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x41, size: 1, disassembly: "MOV B, C", func_symbols: "B <- C", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.b = cpu.c;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x42, size: 1, disassembly: "MOV B, D", func_symbols: "B <- D", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.b = cpu.d;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x43, size: 1, disassembly: "MOV B, E", func_symbols: "B <- E", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.b = cpu.e;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x44, size: 1, disassembly: "MOV B, H", func_symbols: "B <- H", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.b = cpu.h;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x45, size: 1, disassembly: "MOV B, L", func_symbols: "B <- L", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.b = cpu.l;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x46, size: 1, disassembly: "MOV B, M", func_symbols: "B <- (HL)", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let addr = combine_bytes(cpu.h, cpu.l) as usize;   
            cpu.b = cpu.memory[addr];
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x47, size: 1, disassembly: "MOV B, A", func_symbols: "B <- A", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.b = cpu.a;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x48, size: 1, disassembly: "MOV C, B", func_symbols: "C <- B", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.c = cpu.b;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x49, size: 1, disassembly: "MOV C, C", func_symbols: "C <- C", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.c = cpu.c;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x4A, size: 1, disassembly: "MOV C, D", func_symbols: "C <- D", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.c = cpu.d;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x4B, size: 1, disassembly: "MOV C, E", func_symbols: "C <- E", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.c = cpu.e;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x4C, size: 1, disassembly: "MOV C, H", func_symbols: "C <- H", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.c = cpu.h;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x4D, size: 1, disassembly: "MOV C, L", func_symbols: "C <- L", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.c = cpu.l;
        }
    });

    insert_instruction(optable, Instruction { opcode: 0x4E, size: 1, disassembly: "MOV C, M", func_symbols: "C <- (HL)", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let addr = combine_bytes(cpu.h, cpu.l) as usize;
            cpu.c = cpu.memory[addr];
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x50, size: 1, disassembly: "MOV D, B", func_symbols: "D <- B", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.d = cpu.b;
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x51, size: 1, disassembly: "MOV D, C", func_symbols: "D <- C", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.d = cpu.c;
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x52, size: 1, disassembly: "MOV D, D", func_symbols: "D <- D", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.d = cpu.d;
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x53, size: 1, disassembly: "MOV D, E", func_symbols: "D <- E", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.d = cpu.e;
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x54, size: 1, disassembly: "MOV D, H", func_symbols: "D <- H", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.d = cpu.h;
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x55, size: 1, disassembly: "MOV D, L", func_symbols: "D <- L", effected_flags: None,
        func_ptr: |cpu, _, _| { 
          cpu.d = cpu.l;
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x56, size: 1, disassembly: "MOV D, M", func_symbols: "D <- (HL)", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let addr = combine_bytes(cpu.h, cpu.l) as usize;
            cpu.d = cpu.memory[addr];
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x57, size: 1, disassembly: "MOV D, A", func_symbols: "D <- A", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.d = cpu.a
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x58, size: 1, disassembly: "MOV E, B", func_symbols: "E <- B", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.e = cpu.b
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x59, size: 1, disassembly: "MOV E, C", func_symbols: "E <- C", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.e = cpu.c;
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x5A, size: 1, disassembly: "MOV E, D", func_symbols: "E <- D", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.e = cpu.d;
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x5B, size: 1, disassembly: "MOV E, E", func_symbols: "E <- E", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.e = cpu.e;
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x5C, size: 1, disassembly: "MOV E, H", func_symbols: "E <- H", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.e = cpu.h;
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x5D, size: 1, disassembly: "MOV E, L", func_symbols: "E <- L", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.e = cpu.l;
        }
    });
    
    insert_instruction(optable, Instruction { opcode: 0x5E, size: 1, disassembly: "MOV E, M", func_symbols: "E <- (HL)", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let addr = combine_bytes(cpu.h, cpu.l) as usize;            
            cpu.e = cpu.memory[addr];
        }
    });
        
    insert_instruction(optable, Instruction { opcode: 0x5F, size: 1, disassembly: "MOV E, A", func_symbols: "E <- A", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.e = cpu.a;
        }
    });
        
    insert_instruction(optable, Instruction { opcode: 0x60, size: 1, disassembly: "MOV H, B", func_symbols: "H <- B", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.h = cpu.b;
        }
    });
           
    insert_instruction(optable, Instruction { opcode: 0x61, size: 1, disassembly: "MOV H, C", func_symbols: "H <- C", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.h = cpu.c;
        }
    });
        
    insert_instruction(optable, Instruction { opcode: 0x62, size: 1, disassembly: "MOV H, D", func_symbols: "H <- D", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.h = cpu.d;
        }
    });
        
    insert_instruction(optable, Instruction { opcode: 0x63, size: 1, disassembly: "MOV H, E", func_symbols: "H <- E", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.h = cpu.e;
        }
    });
        
    insert_instruction(optable, Instruction { opcode: 0x64, size: 1, disassembly: "MOV H, H", func_symbols: "H <- H", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.h = cpu.h;
        }
    });
        
    insert_instruction(optable, Instruction { opcode: 0x65, size: 1, disassembly: "MOV H, L", func_symbols: "H <- L", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.h = cpu.l;
        }
    });
        
    insert_instruction(optable, Instruction { opcode: 0x66, size: 1, disassembly: "MOV H, M", func_symbols: "H <- (HL)", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let addr = combine_bytes(cpu.h, cpu.l) as usize;
            cpu.h = cpu.memory[addr];
        }
    });
            
    insert_instruction(optable, Instruction { opcode: 0x67, size: 1, disassembly: "MOV H, A", func_symbols: "H <- A", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.h = cpu.a;
        }
    });
            
    insert_instruction(optable, Instruction { opcode: 0x68, size: 1, disassembly: "MOV L, B", func_symbols: "L <- B", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.l = cpu.b;
        }
    });
            
    insert_instruction(optable, Instruction { opcode: 0x69, size: 1, disassembly: "MOV L, C", func_symbols: "L <- C", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.l = cpu.c;
        }
    });
            
    insert_instruction(optable, Instruction { opcode: 0x6A, size: 1, disassembly: "MOV L, D", func_symbols: "L <- D", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.l = cpu.d;
        }
    });
            
    insert_instruction(optable, Instruction { opcode: 0x6A, size: 1, disassembly: "MOV L, D", func_symbols: "L <- D", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.l = cpu.d;
        }
    });
            
    insert_instruction(optable, Instruction { opcode: 0x6B, size: 1, disassembly: "MOV L, E", func_symbols: "L <- E", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.l = cpu.e;
        }
    });
                
    insert_instruction(optable, Instruction { opcode: 0x6C, size: 1, disassembly: "MOV L, H", func_symbols: "L <- H", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.l = cpu.h;
        }
    });
                
    insert_instruction(optable, Instruction { opcode: 0x6D, size: 1, disassembly: "MOV L, L", func_symbols: "L <- L", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.l = cpu.l;
        }
    });
                
    insert_instruction(optable, Instruction { opcode: 0x6E, size: 1, disassembly: "MOV L, M", func_symbols: "L <- (HL)", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            let addr = combine_bytes(cpu.h, cpu.l) as usize;
            cpu.l = cpu.memory[addr];
        }
    });
                
    insert_instruction(optable, Instruction { opcode: 0x6F, size: 1, disassembly: "MOV L, A", func_symbols: "L <- A", effected_flags: None,
        func_ptr: |cpu, _, _| { 
            cpu.l = cpu.a;
        }
    });
}

fn insert_instruction(optable: &mut HashMap<u8, Instruction>, instruction: Instruction) {
    optable.insert(instruction.opcode, instruction);
}

fn set_register_pair(high: &mut u8, low: &mut u8, scalar: u16) {
    *low = scalar as u8;
    *high = (scalar >> 8) as u8;
}


// little endian
const fn combine_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | low as u16
}

// from https://www.tutorialspoint.com/cplusplus-program-to-find-the-parity-of-a-number-efficiently
const fn parity(n: u32) -> bool {
    let mut y = n ^ (n >> 1);
    y = y ^ (y >> 2);
    y = y ^ (y >> 4);
    y = y ^ (y >> 6);
    y = y ^ (y >> 8);
    y = y ^ (y >> 16);
    (y & 1) != 0
}

const fn sign_flag(n: u16) -> bool {
    n & 0x80 != 0
}

const fn carry_flag(n: u16) -> bool {
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
