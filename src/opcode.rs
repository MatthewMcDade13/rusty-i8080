use std::collections::HashMap;
use crate::{Cpu8080, ConditionFlag, combine_bytes, set_byte_pair};
pub struct OpcodeTable(HashMap<u8, Instruction>);

#[derive(Copy, Clone)]
pub struct Instruction {
    pub opcode: u8,
    pub size: u8,
    pub disassembly: &'static str,
    pub mnemonic: &'static str,
    pub effected_flags: Option<&'static str>,
    pub func_ptr: fn(&mut Cpu8080, b2: u8, b3: u8)
}

impl OpcodeTable {

    fn insert(&mut self, instruction: &Instruction) {
        self.0.insert(instruction.opcode, *instruction);
    }

    pub fn get(&self, opcode: u8) -> Option<&Instruction> {
        self.0.get(&opcode)
    }

    pub fn new() -> Self {
        let mut optable = OpcodeTable(HashMap::new());
        optable.insert(&Instruction { opcode: 0x00, size: 1, disassembly: "NOP",       mnemonic: "",                         effected_flags: None, 
        func_ptr: |_, _, _| { } 
        });
        optable.insert(&Instruction { opcode: 0x01, size: 3, disassembly: "LXI B,D16", mnemonic: "B <- byte 3, C <- byte 2", effected_flags: None, 
            func_ptr: |cpu, b2, b3| { cpu.b = b3; cpu.c = b2; } 
        });
        optable.insert(&Instruction { opcode: 0x02, size: 1, disassembly: "STAX B",    mnemonic: "(BC) <- A",                effected_flags: None, 
            func_ptr: |cpu, _, _| { cpu.memory[combine_bytes(cpu.b, cpu.c) as usize] = cpu.a } 
        });

        optable.insert(&Instruction { opcode: 0x03, size: 1, disassembly: "INX B",     mnemonic: "BC <- BC + 1",             effected_flags: None, 
            func_ptr: |cpu, _, _|  { 
                let r = combine_bytes(cpu.b, cpu.c) + 1; 
                set_byte_pair(&mut cpu.b, &mut cpu.c, r) 
            } 
        });
        optable.insert(&Instruction { opcode: 0x04, size: 1, disassembly: "INR B",     mnemonic: "B <- B + 1", effected_flags: "Z,S,P,AC".into(), 
            func_ptr: |cpu, _, _|   { 
                cpu.b = cpu.inr(cpu.b);
            } 
        });

        optable.insert(&Instruction { opcode: 0x05, size: 1, disassembly: "DCR B", mnemonic: "B <- B - 1", effected_flags: "Z,S,P,AC".into(), 
            func_ptr: |cpu, _, _| { 
                cpu.b = cpu.dcr(cpu.b)
            } 
        });

        optable.insert(&Instruction { opcode: 0x06, size: 2, disassembly: "MVI B, D8", mnemonic: "B <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.b = b2;
            } 
        });

        optable.insert(&Instruction { opcode: 0x07, size: 1, disassembly: "RLC", mnemonic: "A = A << 1; bit 0 = prev bit 7; CY = prev bit 7", effected_flags: "CY".into(),
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

        optable.insert(&Instruction { opcode: 0x09, size: 1, disassembly: "DAD B", mnemonic: "HL = HL + BC", effected_flags: "CY".into(),
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l);
                let bc = combine_bytes(cpu.b, cpu.c);
                let result = hl + bc;
                cpu.check_carry(result);

                set_byte_pair(&mut cpu.h, &mut cpu.l, result);
            }
        });

        optable.insert(&Instruction { opcode: 0x0A, size: 1, disassembly: "LDAX B", mnemonic: "A <- (BC)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let bc = combine_bytes(cpu.b, cpu.c) as usize;
                cpu.a = cpu.memory[bc];
            }
        });

        optable.insert(&Instruction { opcode: 0x0B, size: 1, disassembly: "DCX B", mnemonic: "BC = BC-1", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let bc = combine_bytes(cpu.b, cpu.c);
                set_byte_pair(&mut cpu.b, &mut cpu.c, bc - 1);
            }
        });

        optable.insert(&Instruction { opcode: 0x0C, size: 1, disassembly: "INR C", mnemonic: "C <- C + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.c = cpu.inr(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0x0D, size: 1, disassembly: "DCR C", mnemonic: "C <- C - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.c = cpu.dcr(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0x0E, size: 2, disassembly: "MVI C, D8", mnemonic: "C <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.c = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x0F, size: 1, disassembly: "RRC", mnemonic: "A = A >> 1; bit 7 = prev bit 0; CY = prev bit 0", effected_flags: "CY".into(),
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

        optable.insert(&Instruction { opcode: 0x11, size: 3, disassembly: "LXI D,D16", mnemonic: "D <- byte 3, E <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                cpu.d = b3;
                cpu.e = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x12, size: 1, disassembly: "STAX D", mnemonic: "(DE) <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let de = combine_bytes(cpu.d, cpu.e) as usize;
                cpu.memory[de] = cpu.a;
            }
        });

        optable.insert(&Instruction { opcode: 0x13, size: 1, disassembly: "INX D", mnemonic: "DE <- DE + 1", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let result = combine_bytes(cpu.d, cpu.e) + 1;
                set_byte_pair(&mut cpu.d, &mut cpu.e, result);
            }
        });

        optable.insert(&Instruction { opcode: 0x14, size: 1, disassembly: "INR D", mnemonic: "D <- D + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.d = cpu.inr(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0x15, size: 1, disassembly: "DCR D", mnemonic: "D <- D - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.d = cpu.dcr(cpu.d);
        }
    });

        optable.insert(&Instruction { opcode: 0x16, size: 2, disassembly: "MVI D, D8", mnemonic: "D <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.d = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x17, size: 1, disassembly: "RAL", mnemonic: "A = A << 1; bit 0 = prev CY; CY = prev bit 7", effected_flags: "CY".into(),
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

        optable.insert(&Instruction { opcode: 0x19, size: 1, disassembly: "DAD D", mnemonic: "HL = HL + DE", effected_flags: "CY".into(),
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l);
                let de = combine_bytes(cpu.d, cpu.e);
                let result = hl + de;

                cpu.check_carry(result);

                set_byte_pair(&mut cpu.h, &mut cpu.l, result);
            }
        });

        optable.insert(&Instruction { opcode: 0x1A, size: 1, disassembly: "LDAX D", mnemonic: "A <- (DE)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let de = combine_bytes(cpu.d, cpu.e) as usize;
                cpu.a = cpu.memory[de];
            }
        });

        optable.insert(&Instruction { opcode: 0x1B, size: 1, disassembly: "DCX D", mnemonic: "DE <- DE - 1", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let result = combine_bytes(cpu.d, cpu.e) - 1;
                set_byte_pair(&mut cpu.d, &mut cpu.e, result);
            }
        });

        optable.insert(&Instruction { opcode: 0x1C, size: 1, disassembly: "INR E", mnemonic: "E <- E + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.inr(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0x1D, size: 1, disassembly: "DCR E", mnemonic: "E <- E - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.dcr(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0x1E, size: 2, disassembly: "MVI E, D8", mnemonic: "E <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.e = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x1F, size: 1, disassembly: "RAR", mnemonic: "A = A >> 1; bit 7 = prev CY; CY = prev bit 0", effected_flags: "CY".into(),
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

        optable.insert(&Instruction { opcode: 0x21, size: 3, disassembly: "LXI H, D16", mnemonic: "H <- byte 3, L <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                cpu.h = b3;
                cpu.l = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x22, size: 3, disassembly: "SHLD adr", mnemonic: "(adr) <- L; (adr + 1) <- H", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                let addr = combine_bytes(b3, b2) as usize;
                cpu.memory[addr] = cpu.l;
                cpu.memory[addr + 1] = cpu.h;
            }
        });

        optable.insert(&Instruction { opcode: 0x23, size: 1, disassembly: "INX H", mnemonic: "HL <- HL + 1", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let result = combine_bytes(cpu.h, cpu.l);
                set_byte_pair(&mut cpu.h, &mut cpu.l, result + 1);
            }
        });

        optable.insert(&Instruction { opcode: 0x24, size: 1, disassembly: "INR H", mnemonic: "H <- H + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.inr(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0x25, size: 1, disassembly: "DCR H", mnemonic: "H <- H - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.dcr(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0x26, size: 2, disassembly: "MVI H, D8", mnemonic: "H <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.h = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x27, size: 1, disassembly: "DAA", mnemonic: "Decimal Adjust Accumulator", effected_flags: "Z,S,P,CY,AC".into(),
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

        optable.insert(&Instruction { opcode: 0x29, size: 1, disassembly: "DAD H", mnemonic: "HL <- HL + HL", effected_flags: "CY".into(),
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l) as u16;
                let result = hl + hl;
                cpu.check_carry(result);
                set_byte_pair(&mut cpu.h, &mut cpu.l, result);
            }
        });

        optable.insert(&Instruction { opcode: 0x2A, size: 3, disassembly: "LHLD adr", mnemonic: "L <- (adr); H <- (adr + 1)", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                let addr = combine_bytes(b3, b2) as usize;
                cpu.l = cpu.memory[addr];
                cpu.h = cpu.memory[addr + 1];
            }
        });

        optable.insert(&Instruction { opcode: 0x2B, size: 1, disassembly: "DCX H", mnemonic: "HL <- HL - 1", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l);
                set_byte_pair(&mut cpu.h, &mut cpu.l, hl + 1);
            }
        });

        optable.insert(&Instruction { opcode: 0x2C, size: 1, disassembly: "INR L", mnemonic: "L <- L + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.inr(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0x2D, size: 1, disassembly: "DCR L", mnemonic: "L <- L - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.dcr(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0x2E, size: 2, disassembly: "MVI L, D8", mnemonic: "L <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.l = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x2F, size: 1, disassembly: "CMA", mnemonic: "A <- !A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = !cpu.a;
            }
        });

        optable.insert(&Instruction { opcode: 0x31, size: 3, disassembly: "LXI SP, D16", mnemonic: "SP.high <- byte 3; SP.low <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                cpu.sp = combine_bytes(b3, b2); 
            }
        });

        optable.insert(&Instruction { opcode: 0x32, size: 3, disassembly: "STA adr", mnemonic: "(adr) <- A", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                let addr = combine_bytes(b3, b2) as usize;
                cpu.memory[addr] = cpu.a;
            }
        });

        optable.insert(&Instruction { opcode: 0x33, size: 1, disassembly: "INX SP", mnemonic: "SP = SP + 1", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.sp += 1;
            }
        });

        optable.insert(&Instruction { opcode: 0x34, size: 1, disassembly: "INR M", mnemonic: "(HL) <- (HL) + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[hl] = cpu.inr(cpu.memory[hl]);
            }
        });

        optable.insert(&Instruction { opcode: 0x35, size: 1, disassembly: "DCR M", mnemonic: "(HL) <- (HL) - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[hl] = cpu.dcr(cpu.memory[hl]);
            }
        });

        optable.insert(&Instruction { opcode: 0x36, size: 2, disassembly: "MVI M, D8", mnemonic: "(HL) <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                let hl = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[hl] = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x37, size: 1, disassembly: "STC", mnemonic: "CY = 1", effected_flags: "CY".into(),
            func_ptr: |cpu, _, _| { 
                cpu.condition_codes.set(ConditionFlag::Carry);                
            }
        });

        optable.insert(&Instruction { opcode: 0x39, size: 1, disassembly: "DAD SP", mnemonic: "HL <- HL + SP", effected_flags: "CY".into(),
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l);
                let result = hl + cpu.sp;
                cpu.check_carry(result);
                set_byte_pair(&mut cpu.h, &mut cpu.l, result);           
            }
        });

        optable.insert(&Instruction { opcode: 0x3A, size: 3, disassembly: "LDA adr", mnemonic: "A <- (adr)", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                let addr = combine_bytes(b3, b2) as usize;
                cpu.a = cpu.memory[addr];
            }
        });

        optable.insert(&Instruction { opcode: 0x3B, size: 1, disassembly: "DCX SP", mnemonic: "SP <- SP + 1", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                cpu.sp += 1;
            }
        });

        optable.insert(&Instruction { opcode: 0x3C, size: 1, disassembly: "INR A", mnemonic: "A <- A + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.inr(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0x3D, size: 1, disassembly: "DCR A", mnemonic: "A <- A - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.dcr(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0x3E, size: 2, disassembly: "MVI A, D8", mnemonic: "A <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.a = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x3F, size: 1, disassembly: "CMC", mnemonic: "CY = !CY", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                if cpu.condition_codes.is_set(ConditionFlag::Carry) {
                    cpu.condition_codes.unset(ConditionFlag::Carry);
                } else {
                    cpu.condition_codes.set(ConditionFlag::Carry)
                }
            }
        });

        optable.insert(&Instruction { opcode: 0x40, size: 1, disassembly: "MOV B, B", mnemonic: "B <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.b;
            }
        });

        optable.insert(&Instruction { opcode: 0x40, size: 1, disassembly: "MOV B, B", mnemonic: "B <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.b;
            }
        });

        optable.insert(&Instruction { opcode: 0x41, size: 1, disassembly: "MOV B, C", mnemonic: "B <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.c;
            }
        });

        optable.insert(&Instruction { opcode: 0x42, size: 1, disassembly: "MOV B, D", mnemonic: "B <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.d;
            }
        });

        optable.insert(&Instruction { opcode: 0x43, size: 1, disassembly: "MOV B, E", mnemonic: "B <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.e;
            }
        });

        optable.insert(&Instruction { opcode: 0x44, size: 1, disassembly: "MOV B, H", mnemonic: "B <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.h;
            }
        });

        optable.insert(&Instruction { opcode: 0x45, size: 1, disassembly: "MOV B, L", mnemonic: "B <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.l;
            }
        });

        optable.insert(&Instruction { opcode: 0x46, size: 1, disassembly: "MOV B, M", mnemonic: "B <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;   
                cpu.b = cpu.memory[addr];
            }
        });

        optable.insert(&Instruction { opcode: 0x47, size: 1, disassembly: "MOV B, A", mnemonic: "B <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.a;
            }
        });

        optable.insert(&Instruction { opcode: 0x48, size: 1, disassembly: "MOV C, B", mnemonic: "C <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.c = cpu.b;
            }
        });

        optable.insert(&Instruction { opcode: 0x49, size: 1, disassembly: "MOV C, C", mnemonic: "C <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.c = cpu.c;
            }
        });

        optable.insert(&Instruction { opcode: 0x4A, size: 1, disassembly: "MOV C, D", mnemonic: "C <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.c = cpu.d;
            }
        });

        optable.insert(&Instruction { opcode: 0x4B, size: 1, disassembly: "MOV C, E", mnemonic: "C <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.c = cpu.e;
            }
        });

        optable.insert(&Instruction { opcode: 0x4C, size: 1, disassembly: "MOV C, H", mnemonic: "C <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.c = cpu.h;
            }
        });

        optable.insert(&Instruction { opcode: 0x4D, size: 1, disassembly: "MOV C, L", mnemonic: "C <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.c = cpu.l;
            }
        });

        optable.insert(&Instruction { opcode: 0x4E, size: 1, disassembly: "MOV C, M", mnemonic: "C <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.c = cpu.memory[addr];
            }
        });
        
        optable.insert(&Instruction { opcode: 0x50, size: 1, disassembly: "MOV D, B", mnemonic: "D <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.d = cpu.b;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x51, size: 1, disassembly: "MOV D, C", mnemonic: "D <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.d = cpu.c;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x52, size: 1, disassembly: "MOV D, D", mnemonic: "D <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.d = cpu.d;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x53, size: 1, disassembly: "MOV D, E", mnemonic: "D <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.d = cpu.e;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x54, size: 1, disassembly: "MOV D, H", mnemonic: "D <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.d = cpu.h;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x55, size: 1, disassembly: "MOV D, L", mnemonic: "D <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.d = cpu.l;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x56, size: 1, disassembly: "MOV D, M", mnemonic: "D <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.d = cpu.memory[addr];
            }
        });
        
        optable.insert(&Instruction { opcode: 0x57, size: 1, disassembly: "MOV D, A", mnemonic: "D <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.d = cpu.a
            }
        });
        
        optable.insert(&Instruction { opcode: 0x58, size: 1, disassembly: "MOV E, B", mnemonic: "E <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.b
            }
        });
        
        optable.insert(&Instruction { opcode: 0x59, size: 1, disassembly: "MOV E, C", mnemonic: "E <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.c;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x5A, size: 1, disassembly: "MOV E, D", mnemonic: "E <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.d;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x5B, size: 1, disassembly: "MOV E, E", mnemonic: "E <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.e;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x5C, size: 1, disassembly: "MOV E, H", mnemonic: "E <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.h;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x5D, size: 1, disassembly: "MOV E, L", mnemonic: "E <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.l;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x5E, size: 1, disassembly: "MOV E, M", mnemonic: "E <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;            
                cpu.e = cpu.memory[addr];
            }
        });
            
        optable.insert(&Instruction { opcode: 0x5F, size: 1, disassembly: "MOV E, A", mnemonic: "E <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.a;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x60, size: 1, disassembly: "MOV H, B", mnemonic: "H <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.b;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x61, size: 1, disassembly: "MOV H, C", mnemonic: "H <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.c;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x62, size: 1, disassembly: "MOV H, D", mnemonic: "H <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.d;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x63, size: 1, disassembly: "MOV H, E", mnemonic: "H <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.e;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x64, size: 1, disassembly: "MOV H, H", mnemonic: "H <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.h;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x65, size: 1, disassembly: "MOV H, L", mnemonic: "H <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.l;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x66, size: 1, disassembly: "MOV H, M", mnemonic: "H <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.h = cpu.memory[addr];
            }
        });
                
        optable.insert(&Instruction { opcode: 0x67, size: 1, disassembly: "MOV H, A", mnemonic: "H <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.a;
            }
        });
                
        optable.insert(&Instruction { opcode: 0x68, size: 1, disassembly: "MOV L, B", mnemonic: "L <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.b;
            }
        });
                
        optable.insert(&Instruction { opcode: 0x69, size: 1, disassembly: "MOV L, C", mnemonic: "L <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.c;
            }
        });
                
        optable.insert(&Instruction { opcode: 0x6A, size: 1, disassembly: "MOV L, D", mnemonic: "L <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.d;
            }
        });
                
        optable.insert(&Instruction { opcode: 0x6A, size: 1, disassembly: "MOV L, D", mnemonic: "L <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.d;
            }
        });
                
        optable.insert(&Instruction { opcode: 0x6B, size: 1, disassembly: "MOV L, E", mnemonic: "L <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.e;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x6C, size: 1, disassembly: "MOV L, H", mnemonic: "L <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.h;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x6D, size: 1, disassembly: "MOV L, L", mnemonic: "L <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.l;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x6E, size: 1, disassembly: "MOV L, M", mnemonic: "L <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.l = cpu.memory[addr];
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x6F, size: 1, disassembly: "MOV L, A", mnemonic: "L <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.a;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x70, size: 1, disassembly: "MOV M, B", mnemonic: "(HL) <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[addr] = cpu.b;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x72, size: 1, disassembly: "MOV M, D", mnemonic: "(HL) <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[addr] = cpu.d;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x73, size: 1, disassembly: "MOV M, E", mnemonic: "(HL) <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[addr] = cpu.e;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x74, size: 1, disassembly: "MOV M, H", mnemonic: "(HL) <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[addr] = cpu.h;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x76, size: 1, disassembly: "HLT", mnemonic: "Halt - Processor is stopped", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.halted = true;
            }
        });

        optable.insert(&Instruction { opcode: 0x77, size: 1, disassembly: "MOV M, A", mnemonic: "(HL) <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[addr] = cpu.a;
            }
        });

        optable.insert(&Instruction { opcode: 0x78, size: 1, disassembly: "MOV A, B", mnemonic: "A <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.b;
            }
        });

        optable.insert(&Instruction { opcode: 0x79, size: 1, disassembly: "MOV A, C", mnemonic: "A <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.c;
            }
        });

        optable.insert(&Instruction { opcode: 0x7A, size: 1, disassembly: "MOV A, D", mnemonic: "A <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.d;
            }
        });

        optable.insert(&Instruction { opcode: 0x7B, size: 1, disassembly: "MOV A, E", mnemonic: "A <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.e;
            }
        });

        optable.insert(&Instruction { opcode: 0x7C, size: 1, disassembly: "MOV A, H", mnemonic: "A <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.h;
            }
        });

        optable.insert(&Instruction { opcode: 0x7D, size: 1, disassembly: "MOV A, L", mnemonic: "A <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.l;
            }
        });

        optable.insert(&Instruction { opcode: 0x7E, size: 1, disassembly: "MOV A, M", mnemonic: "A <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.a = cpu.memory[addr];
            }
        });
        
        optable.insert(&Instruction { opcode: 0x7F, size: 1, disassembly: "MOV A, A", mnemonic: "A <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.a;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x80, size: 1, disassembly: "ADD B", mnemonic: "A <- A + B", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.b);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x81, size: 1, disassembly: "ADD C", mnemonic: "A <- A + C", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.c);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x82, size: 1, disassembly: "ADD D", mnemonic: "A <- A + D", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.d);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x83, size: 1, disassembly: "ADD E", mnemonic: "A <- A + E", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.e);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x84, size: 1, disassembly: "ADD H", mnemonic: "A <- A + H", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.h);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x85, size: 1, disassembly: "ADD L", mnemonic: "A <- A + L", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.l);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x86, size: 1, disassembly: "ADD M", mnemonic: "A <- A + (HL)", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.add(cpu.memory[addr]);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x87, size: 1, disassembly: "ADD A", mnemonic: "A <- A + A", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.a);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x88, size: 1, disassembly: "ADC B", mnemonic: "A <- A + B + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.b);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x89, size: 1, disassembly: "ADC C", mnemonic: "A <- A + C + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.c);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x8A, size: 1, disassembly: "ADC D", mnemonic: "A <- A + D + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.d);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x8B, size: 1, disassembly: "ADC E", mnemonic: "A <- A + E + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.e);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x8C, size: 1, disassembly: "ADC H", mnemonic: "A <- A + H + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.h);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x8D, size: 1, disassembly: "ADC L", mnemonic: "A <- A + L + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0x8E, size: 1, disassembly: "ADC M", mnemonic: "A <- A + (HL) + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.adc(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0x8F, size: 1, disassembly: "ADC A", mnemonic: "A <- A + A + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0x90, size: 1, disassembly: "SUB B", mnemonic: "A <- A - B", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.b);
            }
        });

        optable.insert(&Instruction { opcode: 0x91, size: 1, disassembly: "SUB C", mnemonic: "A <- A - C", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0x92, size: 1, disassembly: "SUB D", mnemonic: "A <- A - D", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0x93, size: 1, disassembly: "SUB E", mnemonic: "A <- A - E", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0x94, size: 1, disassembly: "SUB H", mnemonic: "A <- A - H", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0x95, size: 1, disassembly: "SUB L", mnemonic: "A <- A - L", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0x96, size: 1, disassembly: "SUB M", mnemonic: "A <- A - (HL)", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.sub(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0x97, size: 1, disassembly: "SUB A", mnemonic: "A <- A - A", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0x98, size: 1, disassembly: "SBB B", mnemonic: "A <- A - B - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.b);
            }
        });

        optable.insert(&Instruction { opcode: 0x99, size: 1, disassembly: "SBB C", mnemonic: "A <- A - C - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0x9A, size: 1, disassembly: "SBB D", mnemonic: "A <- A - D - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0x9B, size: 1, disassembly: "SBB E", mnemonic: "A <- A - E - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0x9C, size: 1, disassembly: "SBB H", mnemonic: "A <- A - H - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0x9D, size: 1, disassembly: "SBB L", mnemonic: "A <- A - L - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0x9E, size: 1, disassembly: "SBB M", mnemonic: "A <- A - (HL) - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.sbb(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0x9F, size: 1, disassembly: "SBB A", mnemonic: "A <- A - A - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0xA0, size: 1, disassembly: "ANA B", mnemonic: "A <- A & B", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.b);
            }
        });

        optable.insert(&Instruction { opcode: 0xA1, size: 1, disassembly: "ANA C", mnemonic: "A <- A & C", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0xA2, size: 1, disassembly: "ANA D", mnemonic: "A <- A & D", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0xA3, size: 1, disassembly: "ANA E", mnemonic: "A <- A & E", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0xA4, size: 1, disassembly: "ANA H", mnemonic: "A <- A & H", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0xA5, size: 1, disassembly: "ANA L", mnemonic: "A <- A & L", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0xA6, size: 1, disassembly: "ANA M", mnemonic: "A <- A & (HL)", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.ana(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0xA7, size: 1, disassembly: "ANA A", mnemonic: "A <- A & A", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0xA8, size: 1, disassembly: "XRA B", mnemonic: "A <- A ^ B", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.b);
            }
        });

        optable.insert(&Instruction { opcode: 0xA9, size: 1, disassembly: "XRA C", mnemonic: "A <- A ^ C", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0xAA, size: 1, disassembly: "XRA D", mnemonic: "A <- A ^ D", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0xAB, size: 1, disassembly: "XRA E", mnemonic: "A <- A ^ E", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0xAC, size: 1, disassembly: "XRA H", mnemonic: "A <- A ^ H", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0xAD, size: 1, disassembly: "XRA L", mnemonic: "A <- A ^ L", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0xAE, size: 1, disassembly: "XRA M", mnemonic: "A <- A ^ (HL)", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.xra(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0xAF, size: 1, disassembly: "XRA A", mnemonic: "A <- A ^ A", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0xB0, size: 1, disassembly: "ORA B", mnemonic: "A <- A | B", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.b);
            }
        });

        optable.insert(&Instruction { opcode: 0xB1, size: 1, disassembly: "ORA C", mnemonic: "A <- A | C", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0xB2, size: 1, disassembly: "ORA D", mnemonic: "A <- A | D", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0xB3, size: 1, disassembly: "ORA E", mnemonic: "A <- A | E", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0xB4, size: 1, disassembly: "ORA H", mnemonic: "A <- A | H", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0xB5, size: 1, disassembly: "ORA L", mnemonic: "A <- A | L", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0xB6, size: 1, disassembly: "ORA M", mnemonic: "A <- A | (HL)", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.ora(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0xB7, size: 1, disassembly: "ORA A", mnemonic: "A <- A | A", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0xB8, size: 1, disassembly: "CMP B", mnemonic: "A - B", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.b);
            }
        });

        optable.insert(&Instruction { opcode: 0xB9, size: 1, disassembly: "CMP C", mnemonic: "A - C", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0xBA, size: 1, disassembly: "CMP D", mnemonic: "A - D", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0xBB, size: 1, disassembly: "CMP E", mnemonic: "A - E", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0xBC, size: 1, disassembly: "CMP H", mnemonic: "A - H", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0xBD, size: 1, disassembly: "CMP L", mnemonic: "A - L", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0xBE, size: 1, disassembly: "CMP M", mnemonic: "A - (HL)", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.cmp(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0xBF, size: 1, disassembly: "CMP A", mnemonic: "A - A", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0xC0, size: 1, disassembly: "RNZ", mnemonic: "if NZ, RET", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                if !cpu.condition_codes.is_set(ConditionFlag::Zero) {
                    cpu.ret();
                }
            }
        });

        optable.insert(&Instruction { opcode: 0xC1, size: 1, disassembly: "POP B", mnemonic: "C <- (sp); B <- (sp + 1); sp <- sp + 2", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let (high, low) = cpu.pop_stack_parts();
                cpu.b = high;
                cpu.c = low;
            }
        });

        optable.insert(&Instruction { opcode: 0xC2, size: 3, disassembly: "JNZ adr", mnemonic: "if NZ, PC <- adr", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                if !cpu.condition_codes.is_set(ConditionFlag::Zero) {
                    let addr = combine_bytes(b3, b2);
                    cpu.jmp(addr);
                }
            }
        });

        optable.insert(&Instruction { opcode: 0xC3, size: 3, disassembly: "JMP adr", mnemonic: "PC <- adr", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                let addr = combine_bytes(b3, b2);
                cpu.jmp(addr);
            }
        });

        optable.insert(&Instruction { opcode: 0xC4, size: 3, disassembly: "CNZ adr", mnemonic: "if NZ, CALL adr", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                if !cpu.condition_codes.is_set(ConditionFlag::Zero) {
                    let addr = combine_bytes(b3, b2);
                    cpu.call(addr);
                }
            }
        });
        
        optable.insert(&Instruction { opcode: 0xC5, size: 1, disassembly: "PUSH B", mnemonic: "(sp-2) <- C; (sp-1) <- B; sp <- sp-2", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let val = combine_bytes(cpu.b, cpu.c);
                cpu.push_stack(val);
            }
        });

        optable.insert(&Instruction { opcode: 0xC6, size: 2, disassembly: "ADI D8", mnemonic: "A <- A + byte", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, b2, _| { 
                cpu.add(b2);
            }
        });
        
        optable.insert(&Instruction { opcode: 0xC7, size: 1, disassembly: "RST 0", mnemonic: "CALL $0", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.call(0x00);
            }
        });

        optable.insert(&Instruction { opcode: 0xC8, size: 1, disassembly: "RZ", mnemonic: "if Z, RET", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                if cpu.condition_codes.is_set(ConditionFlag::Zero) {
                    cpu.ret();
                }
            }
        });

        optable
    }
}

