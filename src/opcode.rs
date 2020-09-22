use std::collections::HashMap;
use crate::{Cpu8080, ConditionFlag, combine_bytes, set_byte_pair};
pub struct OpcodeTable(HashMap<u8, Instruction>);

#[derive(Copy, Clone)]
pub struct Instruction {
    pub opcode: u8,
    pub size: u8,
    pub disassembly: &'static str,
    pub func_symbols: &'static str,
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
        optable.insert(&Instruction { opcode: 0x00, size: 1, disassembly: "NOP",       func_symbols: "",                         effected_flags: None, 
        func_ptr: |_, _, _| { } 
        });
        optable.insert(&Instruction { opcode: 0x01, size: 3, disassembly: "LXI B,D16", func_symbols: "B <- byte 3, C <- byte 2", effected_flags: None, 
            func_ptr: |cpu, b2, b3| { cpu.b = b3; cpu.c = b2; } 
        });
        optable.insert(&Instruction { opcode: 0x02, size: 1, disassembly: "STAX B",    func_symbols: "(BC) <- A",                effected_flags: None, 
            func_ptr: |cpu, _, _| { cpu.memory[combine_bytes(cpu.b, cpu.c) as usize] = cpu.a } 
        });

        optable.insert(&Instruction { opcode: 0x03, size: 1, disassembly: "INX B",     func_symbols: "BC <- BC + 1",             effected_flags: None, 
            func_ptr: |cpu, _, _|  { 
                let r = combine_bytes(cpu.b, cpu.c) + 1; 
                set_byte_pair(&mut cpu.b, &mut cpu.c, r) 
            } 
        });
        optable.insert(&Instruction { opcode: 0x04, size: 1, disassembly: "INR B",     func_symbols: "B <- B + 1", effected_flags: "Z,S,P,AC".into(), 
            func_ptr: |cpu, _, _|   { 
                cpu.b = cpu.inr(cpu.b);
            } 
        });

        optable.insert(&Instruction { opcode: 0x05, size: 1, disassembly: "DCR B", func_symbols: "B <- B - 1", effected_flags: "Z,S,P,AC".into(), 
            func_ptr: |cpu, _, _| { 
                cpu.b = cpu.dcr(cpu.b)
            } 
        });

        optable.insert(&Instruction { opcode: 0x06, size: 2, disassembly: "MVI B, D8", func_symbols: "B <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.b = b2;
            } 
        });

        optable.insert(&Instruction { opcode: 0x07, size: 1, disassembly: "RLC", func_symbols: "A = A << 1; bit 0 = prev bit 7; CY = prev bit 7", effected_flags: "CY".into(),
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

        optable.insert(&Instruction { opcode: 0x09, size: 1, disassembly: "DAD B", func_symbols: "HL = HL + BC", effected_flags: "CY".into(),
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l);
                let bc = combine_bytes(cpu.b, cpu.c);
                let result = hl + bc;
                cpu.check_carry(result);

                set_byte_pair(&mut cpu.h, &mut cpu.l, result);
            }
        });

        optable.insert(&Instruction { opcode: 0x0A, size: 1, disassembly: "LDAX B", func_symbols: "A <- (BC)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let bc = combine_bytes(cpu.b, cpu.c) as usize;
                cpu.a = cpu.memory[bc];
            }
        });

        optable.insert(&Instruction { opcode: 0x0B, size: 1, disassembly: "DCX B", func_symbols: "BC = BC-1", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let bc = combine_bytes(cpu.b, cpu.c);
                set_byte_pair(&mut cpu.b, &mut cpu.c, bc - 1);
            }
        });

        optable.insert(&Instruction { opcode: 0x0C, size: 1, disassembly: "INR C", func_symbols: "C <- C + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.c = cpu.inr(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0x0D, size: 1, disassembly: "DCR C", func_symbols: "C <- C - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.c = cpu.dcr(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0x0E, size: 2, disassembly: "MVI C, D8", func_symbols: "C <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.c = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x0F, size: 1, disassembly: "RRC", func_symbols: "A = A >> 1; bit 7 = prev bit 0; CY = prev bit 0", effected_flags: "CY".into(),
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

        optable.insert(&Instruction { opcode: 0x11, size: 3, disassembly: "LXI D,D16", func_symbols: "D <- byte 3, E <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                cpu.d = b3;
                cpu.e = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x12, size: 1, disassembly: "STAX D", func_symbols: "(DE) <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let de = combine_bytes(cpu.d, cpu.e) as usize;
                cpu.memory[de] = cpu.a;
            }
        });

        optable.insert(&Instruction { opcode: 0x13, size: 1, disassembly: "INX D", func_symbols: "DE <- DE + 1", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let result = combine_bytes(cpu.d, cpu.e) + 1;
                set_byte_pair(&mut cpu.d, &mut cpu.e, result);
            }
        });

        optable.insert(&Instruction { opcode: 0x14, size: 1, disassembly: "INR D", func_symbols: "D <- D + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.d = cpu.inr(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0x15, size: 1, disassembly: "DCR D", func_symbols: "D <- D - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.d = cpu.dcr(cpu.d);
        }
    });

        optable.insert(&Instruction { opcode: 0x16, size: 2, disassembly: "MVI D, D8", func_symbols: "D <- byte 2", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let result = cpu.d as u16 - 1;
                cpu.check_zero(result);
                cpu.check_sign(result);
                cpu.check_parity(result as u32);
                // aux carry
                cpu.d = result as u8;
            }
        });

        optable.insert(&Instruction { opcode: 0x17, size: 1, disassembly: "RAL", func_symbols: "A = A << 1; bit 0 = prev CY; CY = prev bit 7", effected_flags: "CY".into(),
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

        optable.insert(&Instruction { opcode: 0x19, size: 1, disassembly: "DAD D", func_symbols: "HL = HL + DE", effected_flags: "CY".into(),
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l);
                let de = combine_bytes(cpu.d, cpu.e);
                let result = hl + de;

                cpu.check_carry(result);

                set_byte_pair(&mut cpu.h, &mut cpu.l, result);
            }
        });

        optable.insert(&Instruction { opcode: 0x1A, size: 1, disassembly: "LDAX D", func_symbols: "A <- (DE)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let de = combine_bytes(cpu.d, cpu.e) as usize;
                cpu.a = cpu.memory[de];
            }
        });

        optable.insert(&Instruction { opcode: 0x1B, size: 1, disassembly: "DCX D", func_symbols: "DE <- DE - 1", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let result = combine_bytes(cpu.d, cpu.e) - 1;
                set_byte_pair(&mut cpu.d, &mut cpu.e, result);
            }
        });

        optable.insert(&Instruction { opcode: 0x1C, size: 1, disassembly: "INR E", func_symbols: "E <- E + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.inr(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0x1D, size: 1, disassembly: "DCR E", func_symbols: "E <- E - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.dcr(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0x1E, size: 2, disassembly: "MVI E, D8", func_symbols: "E <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.e = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x1F, size: 1, disassembly: "RAR", func_symbols: "A = A >> 1; bit 7 = prev CY; CY = prev bit 0", effected_flags: "CY".into(),
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

        optable.insert(&Instruction { opcode: 0x21, size: 3, disassembly: "LXI H, D16", func_symbols: "H <- byte 3, L <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                cpu.h = b3;
                cpu.l = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x22, size: 3, disassembly: "SHLD adr", func_symbols: "(adr) <- L; (adr + 1) <- H", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                let addr = combine_bytes(b3, b2) as usize;
                cpu.memory[addr] = cpu.l;
                cpu.memory[addr + 1] = cpu.h;
            }
        });

        optable.insert(&Instruction { opcode: 0x23, size: 1, disassembly: "INX H", func_symbols: "HL <- HL + 1", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let result = combine_bytes(cpu.h, cpu.l);
                set_byte_pair(&mut cpu.h, &mut cpu.l, result + 1);
            }
        });

        optable.insert(&Instruction { opcode: 0x24, size: 1, disassembly: "INR H", func_symbols: "H <- H + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.inr(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0x25, size: 1, disassembly: "DCR H", func_symbols: "H <- H - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.dcr(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0x26, size: 2, disassembly: "MVI H, D8", func_symbols: "H <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.h = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x27, size: 1, disassembly: "DAA", func_symbols: "Decimal Adjust Accumulator", effected_flags: "Z,S,P,CY,AC".into(),
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

        optable.insert(&Instruction { opcode: 0x29, size: 1, disassembly: "DAD H", func_symbols: "HL <- HL + HL", effected_flags: "CY".into(),
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l) as u16;
                let result = hl + hl;
                cpu.check_carry(result);
                set_byte_pair(&mut cpu.h, &mut cpu.l, result);
            }
        });

        optable.insert(&Instruction { opcode: 0x2A, size: 3, disassembly: "LHLD adr", func_symbols: "L <- (adr); H <- (adr + 1)", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                let addr = combine_bytes(b3, b2) as usize;
                cpu.l = cpu.memory[addr];
                cpu.h = cpu.memory[addr + 1];
            }
        });

        optable.insert(&Instruction { opcode: 0x2B, size: 1, disassembly: "DCX H", func_symbols: "HL <- HL - 1", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l);
                set_byte_pair(&mut cpu.h, &mut cpu.l, hl + 1);
            }
        });

        optable.insert(&Instruction { opcode: 0x2C, size: 1, disassembly: "INR L", func_symbols: "L <- L + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.inr(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0x2D, size: 1, disassembly: "DCR L", func_symbols: "L <- L - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.dcr(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0x2E, size: 2, disassembly: "MVI L, D8", func_symbols: "L <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.l = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x2F, size: 1, disassembly: "CMA", func_symbols: "A <- !A", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.a = !cpu.a;
            }
        });

        optable.insert(&Instruction { opcode: 0x31, size: 3, disassembly: "LXI SP, D16", func_symbols: "SP.high <- byte 3; SP.low <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                cpu.sp = combine_bytes(b3, b2); 
            }
        });

        optable.insert(&Instruction { opcode: 0x32, size: 3, disassembly: "STA adr", func_symbols: "(adr) <- A", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                let addr = combine_bytes(b3, b2) as usize;
                cpu.memory[addr] = cpu.a;
            }
        });

        optable.insert(&Instruction { opcode: 0x33, size: 1, disassembly: "INX SP", func_symbols: "SP = SP + 1", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.sp += 1;
            }
        });

        optable.insert(&Instruction { opcode: 0x34, size: 1, disassembly: "INR M", func_symbols: "(HL) <- (HL) + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[hl] = cpu.inr(cpu.memory[hl]);
            }
        });

        optable.insert(&Instruction { opcode: 0x35, size: 1, disassembly: "DCR M", func_symbols: "(HL) <- (HL) - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[hl] = cpu.dcr(cpu.memory[hl]);
            }
        });

        optable.insert(&Instruction { opcode: 0x36, size: 2, disassembly: "MVI M, D8", func_symbols: "(HL) <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                let hl = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[hl] = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x37, size: 1, disassembly: "STC", func_symbols: "CY = 1", effected_flags: "CY".into(),
            func_ptr: |cpu, _, _| { 
                cpu.condition_codes.set(ConditionFlag::Carry);                
            }
        });

        optable.insert(&Instruction { opcode: 0x39, size: 1, disassembly: "DAD SP", func_symbols: "HL <- HL + SP", effected_flags: "CY".into(),
            func_ptr: |cpu, _, _| { 
                let hl = combine_bytes(cpu.h, cpu.l);
                let result = hl + cpu.sp;
                cpu.check_carry(result);
                set_byte_pair(&mut cpu.h, &mut cpu.l, result);           
            }
        });

        optable.insert(&Instruction { opcode: 0x3A, size: 3, disassembly: "LDA adr", func_symbols: "A <- (adr)", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                let addr = combine_bytes(b3, b2) as usize;
                cpu.a = cpu.memory[addr];
            }
        });

        optable.insert(&Instruction { opcode: 0x3B, size: 1, disassembly: "DCX SP", func_symbols: "SP <- SP + 1", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                cpu.sp += 1;
            }
        });

        optable.insert(&Instruction { opcode: 0x3C, size: 1, disassembly: "INR A", func_symbols: "A <- A + 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.inr(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0x3D, size: 1, disassembly: "DCR A", func_symbols: "A <- A - 1", effected_flags: "Z,S,P,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.dcr(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0x3E, size: 2, disassembly: "MVI A, D8", func_symbols: "A <- byte 2", effected_flags: None,
            func_ptr: |cpu, b2, _| { 
                cpu.a = b2;
            }
        });

        optable.insert(&Instruction { opcode: 0x3F, size: 1, disassembly: "CMC", func_symbols: "CY = !CY", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                if cpu.condition_codes.is_set(ConditionFlag::Carry) {
                    cpu.condition_codes.unset(ConditionFlag::Carry);
                } else {
                    cpu.condition_codes.set(ConditionFlag::Carry)
                }
            }
        });

        optable.insert(&Instruction { opcode: 0x40, size: 1, disassembly: "MOV B, B", func_symbols: "B <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.b;
            }
        });

        optable.insert(&Instruction { opcode: 0x40, size: 1, disassembly: "MOV B, B", func_symbols: "B <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.b;
            }
        });

        optable.insert(&Instruction { opcode: 0x41, size: 1, disassembly: "MOV B, C", func_symbols: "B <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.c;
            }
        });

        optable.insert(&Instruction { opcode: 0x42, size: 1, disassembly: "MOV B, D", func_symbols: "B <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.d;
            }
        });

        optable.insert(&Instruction { opcode: 0x43, size: 1, disassembly: "MOV B, E", func_symbols: "B <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.e;
            }
        });

        optable.insert(&Instruction { opcode: 0x44, size: 1, disassembly: "MOV B, H", func_symbols: "B <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.h;
            }
        });

        optable.insert(&Instruction { opcode: 0x45, size: 1, disassembly: "MOV B, L", func_symbols: "B <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.l;
            }
        });

        optable.insert(&Instruction { opcode: 0x46, size: 1, disassembly: "MOV B, M", func_symbols: "B <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;   
                cpu.b = cpu.memory[addr];
            }
        });

        optable.insert(&Instruction { opcode: 0x47, size: 1, disassembly: "MOV B, A", func_symbols: "B <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.b = cpu.a;
            }
        });

        optable.insert(&Instruction { opcode: 0x48, size: 1, disassembly: "MOV C, B", func_symbols: "C <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.c = cpu.b;
            }
        });

        optable.insert(&Instruction { opcode: 0x49, size: 1, disassembly: "MOV C, C", func_symbols: "C <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.c = cpu.c;
            }
        });

        optable.insert(&Instruction { opcode: 0x4A, size: 1, disassembly: "MOV C, D", func_symbols: "C <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.c = cpu.d;
            }
        });

        optable.insert(&Instruction { opcode: 0x4B, size: 1, disassembly: "MOV C, E", func_symbols: "C <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.c = cpu.e;
            }
        });

        optable.insert(&Instruction { opcode: 0x4C, size: 1, disassembly: "MOV C, H", func_symbols: "C <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.c = cpu.h;
            }
        });

        optable.insert(&Instruction { opcode: 0x4D, size: 1, disassembly: "MOV C, L", func_symbols: "C <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.c = cpu.l;
            }
        });

        optable.insert(&Instruction { opcode: 0x4E, size: 1, disassembly: "MOV C, M", func_symbols: "C <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.c = cpu.memory[addr];
            }
        });
        
        optable.insert(&Instruction { opcode: 0x50, size: 1, disassembly: "MOV D, B", func_symbols: "D <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.d = cpu.b;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x51, size: 1, disassembly: "MOV D, C", func_symbols: "D <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.d = cpu.c;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x52, size: 1, disassembly: "MOV D, D", func_symbols: "D <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.d = cpu.d;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x53, size: 1, disassembly: "MOV D, E", func_symbols: "D <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.d = cpu.e;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x54, size: 1, disassembly: "MOV D, H", func_symbols: "D <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.d = cpu.h;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x55, size: 1, disassembly: "MOV D, L", func_symbols: "D <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
            cpu.d = cpu.l;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x56, size: 1, disassembly: "MOV D, M", func_symbols: "D <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.d = cpu.memory[addr];
            }
        });
        
        optable.insert(&Instruction { opcode: 0x57, size: 1, disassembly: "MOV D, A", func_symbols: "D <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.d = cpu.a
            }
        });
        
        optable.insert(&Instruction { opcode: 0x58, size: 1, disassembly: "MOV E, B", func_symbols: "E <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.b
            }
        });
        
        optable.insert(&Instruction { opcode: 0x59, size: 1, disassembly: "MOV E, C", func_symbols: "E <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.c;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x5A, size: 1, disassembly: "MOV E, D", func_symbols: "E <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.d;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x5B, size: 1, disassembly: "MOV E, E", func_symbols: "E <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.e;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x5C, size: 1, disassembly: "MOV E, H", func_symbols: "E <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.h;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x5D, size: 1, disassembly: "MOV E, L", func_symbols: "E <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.l;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x5E, size: 1, disassembly: "MOV E, M", func_symbols: "E <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;            
                cpu.e = cpu.memory[addr];
            }
        });
            
        optable.insert(&Instruction { opcode: 0x5F, size: 1, disassembly: "MOV E, A", func_symbols: "E <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.e = cpu.a;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x60, size: 1, disassembly: "MOV H, B", func_symbols: "H <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.b;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x61, size: 1, disassembly: "MOV H, C", func_symbols: "H <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.c;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x62, size: 1, disassembly: "MOV H, D", func_symbols: "H <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.d;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x63, size: 1, disassembly: "MOV H, E", func_symbols: "H <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.e;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x64, size: 1, disassembly: "MOV H, H", func_symbols: "H <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.h;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x65, size: 1, disassembly: "MOV H, L", func_symbols: "H <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.l;
            }
        });
            
        optable.insert(&Instruction { opcode: 0x66, size: 1, disassembly: "MOV H, M", func_symbols: "H <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.h = cpu.memory[addr];
            }
        });
                
        optable.insert(&Instruction { opcode: 0x67, size: 1, disassembly: "MOV H, A", func_symbols: "H <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.h = cpu.a;
            }
        });
                
        optable.insert(&Instruction { opcode: 0x68, size: 1, disassembly: "MOV L, B", func_symbols: "L <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.b;
            }
        });
                
        optable.insert(&Instruction { opcode: 0x69, size: 1, disassembly: "MOV L, C", func_symbols: "L <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.c;
            }
        });
                
        optable.insert(&Instruction { opcode: 0x6A, size: 1, disassembly: "MOV L, D", func_symbols: "L <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.d;
            }
        });
                
        optable.insert(&Instruction { opcode: 0x6A, size: 1, disassembly: "MOV L, D", func_symbols: "L <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.d;
            }
        });
                
        optable.insert(&Instruction { opcode: 0x6B, size: 1, disassembly: "MOV L, E", func_symbols: "L <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.e;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x6C, size: 1, disassembly: "MOV L, H", func_symbols: "L <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.h;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x6D, size: 1, disassembly: "MOV L, L", func_symbols: "L <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.l;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x6E, size: 1, disassembly: "MOV L, M", func_symbols: "L <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.l = cpu.memory[addr];
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x6F, size: 1, disassembly: "MOV L, A", func_symbols: "L <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.l = cpu.a;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x70, size: 1, disassembly: "MOV M, B", func_symbols: "(HL) <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[addr] = cpu.b;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x72, size: 1, disassembly: "MOV M, D", func_symbols: "(HL) <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[addr] = cpu.d;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x73, size: 1, disassembly: "MOV M, E", func_symbols: "(HL) <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[addr] = cpu.e;
            }
        });
                    
        optable.insert(&Instruction { opcode: 0x74, size: 1, disassembly: "MOV M, H", func_symbols: "(HL) <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[addr] = cpu.h;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x76, size: 1, disassembly: "HLT", func_symbols: "Halt - Processor is stopped", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.halted = true;
            }
        });

        optable.insert(&Instruction { opcode: 0x77, size: 1, disassembly: "MOV M, A", func_symbols: "(HL) <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.memory[addr] = cpu.a;
            }
        });

        optable.insert(&Instruction { opcode: 0x78, size: 1, disassembly: "MOV A, B", func_symbols: "A <- B", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.b;
            }
        });

        optable.insert(&Instruction { opcode: 0x79, size: 1, disassembly: "MOV A, C", func_symbols: "A <- C", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.c;
            }
        });

        optable.insert(&Instruction { opcode: 0x7A, size: 1, disassembly: "MOV A, D", func_symbols: "A <- D", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.d;
            }
        });

        optable.insert(&Instruction { opcode: 0x7B, size: 1, disassembly: "MOV A, E", func_symbols: "A <- E", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.e;
            }
        });

        optable.insert(&Instruction { opcode: 0x7C, size: 1, disassembly: "MOV A, H", func_symbols: "A <- H", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.h;
            }
        });

        optable.insert(&Instruction { opcode: 0x7D, size: 1, disassembly: "MOV A, L", func_symbols: "A <- L", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.l;
            }
        });

        optable.insert(&Instruction { opcode: 0x7E, size: 1, disassembly: "MOV A, M", func_symbols: "A <- (HL)", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.a = cpu.memory[addr];
            }
        });
        
        optable.insert(&Instruction { opcode: 0x7F, size: 1, disassembly: "MOV A, A", func_symbols: "A <- A", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.a = cpu.a;
            }
        });
        
        optable.insert(&Instruction { opcode: 0x80, size: 1, disassembly: "ADD B", func_symbols: "A <- A + B", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.b);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x81, size: 1, disassembly: "ADD C", func_symbols: "A <- A + C", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.c);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x82, size: 1, disassembly: "ADD D", func_symbols: "A <- A + D", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.d);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x83, size: 1, disassembly: "ADD E", func_symbols: "A <- A + E", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.e);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x84, size: 1, disassembly: "ADD H", func_symbols: "A <- A + H", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.h);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x85, size: 1, disassembly: "ADD L", func_symbols: "A <- A + L", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.l);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x86, size: 1, disassembly: "ADD M", func_symbols: "A <- A + (HL)", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.add(cpu.memory[addr]);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x87, size: 1, disassembly: "ADD A", func_symbols: "A <- A + A", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.add(cpu.a);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x88, size: 1, disassembly: "ADC B", func_symbols: "A <- A + B + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.b);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x89, size: 1, disassembly: "ADC C", func_symbols: "A <- A + C + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.c);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x8A, size: 1, disassembly: "ADC D", func_symbols: "A <- A + D + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.d);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x8B, size: 1, disassembly: "ADC E", func_symbols: "A <- A + E + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.e);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x8C, size: 1, disassembly: "ADC H", func_symbols: "A <- A + H + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.h);
            }
        });
        
        optable.insert(&Instruction { opcode: 0x8D, size: 1, disassembly: "ADC L", func_symbols: "A <- A + L + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0x8E, size: 1, disassembly: "ADC M", func_symbols: "A <- A + (HL) + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.adc(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0x8F, size: 1, disassembly: "ADC A", func_symbols: "A <- A + A + CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.adc(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0x90, size: 1, disassembly: "SUB B", func_symbols: "A <- A - B", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.b);
            }
        });

        optable.insert(&Instruction { opcode: 0x91, size: 1, disassembly: "SUB C", func_symbols: "A <- A - C", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0x92, size: 1, disassembly: "SUB D", func_symbols: "A <- A - D", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0x93, size: 1, disassembly: "SUB E", func_symbols: "A <- A - E", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0x94, size: 1, disassembly: "SUB H", func_symbols: "A <- A - H", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0x95, size: 1, disassembly: "SUB L", func_symbols: "A <- A - L", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0x96, size: 1, disassembly: "SUB M", func_symbols: "A <- A - (HL)", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.sub(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0x97, size: 1, disassembly: "SUB A", func_symbols: "A <- A - A", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sub(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0x98, size: 1, disassembly: "SBB B", func_symbols: "A <- A - B - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.b);
            }
        });

        optable.insert(&Instruction { opcode: 0x99, size: 1, disassembly: "SBB C", func_symbols: "A <- A - C - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0x9A, size: 1, disassembly: "SBB D", func_symbols: "A <- A - D - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0x9B, size: 1, disassembly: "SBB E", func_symbols: "A <- A - E - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0x9C, size: 1, disassembly: "SBB H", func_symbols: "A <- A - H - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0x9D, size: 1, disassembly: "SBB L", func_symbols: "A <- A - L - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0x9E, size: 1, disassembly: "SBB M", func_symbols: "A <- A - (HL) - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.sbb(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0x9F, size: 1, disassembly: "SBB A", func_symbols: "A <- A - A - CY", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.sbb(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0xA0, size: 1, disassembly: "ANA B", func_symbols: "A <- A & B", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.b);
            }
        });

        optable.insert(&Instruction { opcode: 0xA1, size: 1, disassembly: "ANA C", func_symbols: "A <- A & C", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0xA2, size: 1, disassembly: "ANA D", func_symbols: "A <- A & D", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0xA3, size: 1, disassembly: "ANA E", func_symbols: "A <- A & E", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0xA4, size: 1, disassembly: "ANA H", func_symbols: "A <- A & H", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0xA5, size: 1, disassembly: "ANA L", func_symbols: "A <- A & L", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0xA6, size: 1, disassembly: "ANA M", func_symbols: "A <- A & (HL)", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.ana(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0xA7, size: 1, disassembly: "ANA A", func_symbols: "A <- A & A", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ana(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0xA8, size: 1, disassembly: "XRA B", func_symbols: "A <- A ^ B", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.b);
            }
        });

        optable.insert(&Instruction { opcode: 0xA9, size: 1, disassembly: "XRA C", func_symbols: "A <- A ^ C", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0xAA, size: 1, disassembly: "XRA D", func_symbols: "A <- A ^ D", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0xAB, size: 1, disassembly: "XRA E", func_symbols: "A <- A ^ E", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0xAC, size: 1, disassembly: "XRA H", func_symbols: "A <- A ^ H", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0xAD, size: 1, disassembly: "XRA L", func_symbols: "A <- A ^ L", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0xAE, size: 1, disassembly: "XRA M", func_symbols: "A <- A ^ (HL)", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.xra(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0xAF, size: 1, disassembly: "XRA A", func_symbols: "A <- A ^ A", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.xra(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0xB0, size: 1, disassembly: "ORA B", func_symbols: "A <- A | B", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.b);
            }
        });

        optable.insert(&Instruction { opcode: 0xB1, size: 1, disassembly: "ORA C", func_symbols: "A <- A | C", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0xB2, size: 1, disassembly: "ORA D", func_symbols: "A <- A | D", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0xB3, size: 1, disassembly: "ORA E", func_symbols: "A <- A | E", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0xB4, size: 1, disassembly: "ORA H", func_symbols: "A <- A | H", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0xB5, size: 1, disassembly: "ORA L", func_symbols: "A <- A | L", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0xB6, size: 1, disassembly: "ORA M", func_symbols: "A <- A | (HL)", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.ora(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0xB7, size: 1, disassembly: "ORA A", func_symbols: "A <- A | A", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.ora(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0xB8, size: 1, disassembly: "CMP B", func_symbols: "A - B", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.b);
            }
        });

        optable.insert(&Instruction { opcode: 0xB9, size: 1, disassembly: "CMP C", func_symbols: "A - C", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.c);
            }
        });

        optable.insert(&Instruction { opcode: 0xBA, size: 1, disassembly: "CMP D", func_symbols: "A - D", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.d);
            }
        });

        optable.insert(&Instruction { opcode: 0xBB, size: 1, disassembly: "CMP E", func_symbols: "A - E", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.e);
            }
        });

        optable.insert(&Instruction { opcode: 0xBC, size: 1, disassembly: "CMP H", func_symbols: "A - H", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.h);
            }
        });

        optable.insert(&Instruction { opcode: 0xBD, size: 1, disassembly: "CMP L", func_symbols: "A - L", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.l);
            }
        });

        optable.insert(&Instruction { opcode: 0xBE, size: 1, disassembly: "CMP M", func_symbols: "A - (HL)", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                let addr = combine_bytes(cpu.h, cpu.l) as usize;
                cpu.cmp(cpu.memory[addr]);
            }
        });

        optable.insert(&Instruction { opcode: 0xBF, size: 1, disassembly: "CMP A", func_symbols: "A - A", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, _, _| { 
                cpu.cmp(cpu.a);
            }
        });

        optable.insert(&Instruction { opcode: 0xC0, size: 1, disassembly: "RNZ", func_symbols: "if NZ, RET", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                if !cpu.condition_codes.is_set(ConditionFlag::Zero) {
                    cpu.ret();
                }
            }
        });

        optable.insert(&Instruction { opcode: 0xC1, size: 1, disassembly: "POP B", func_symbols: "C <- (sp); B <- (sp + 1); sp <- sp + 2", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let (high, low) = cpu.pop_stack_parts();
                cpu.b = high;
                cpu.c = low;
            }
        });

        optable.insert(&Instruction { opcode: 0xC2, size: 3, disassembly: "JNZ adr", func_symbols: "if NZ, PC <- adr", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                if !cpu.condition_codes.is_set(ConditionFlag::Zero) {
                    let addr = combine_bytes(b3, b2);
                    cpu.jmp(addr);
                }
            }
        });

        optable.insert(&Instruction { opcode: 0xC3, size: 3, disassembly: "JMP adr", func_symbols: "PC <- adr", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                let addr = combine_bytes(b3, b2);
                cpu.jmp(addr);
            }
        });

        optable.insert(&Instruction { opcode: 0xC4, size: 3, disassembly: "CNZ adr", func_symbols: "if NZ, CALL adr", effected_flags: None,
            func_ptr: |cpu, b2, b3| { 
                if !cpu.condition_codes.is_set(ConditionFlag::Zero) {
                    let addr = combine_bytes(b3, b2);
                    cpu.call(addr);
                }
            }
        });
        
        optable.insert(&Instruction { opcode: 0xC5, size: 1, disassembly: "PUSH B", func_symbols: "(sp-2) <- C; (sp-1) <- B; sp <- sp-2", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                let val = combine_bytes(cpu.b, cpu.c);
                cpu.push_stack(val);
            }
        });

        optable.insert(&Instruction { opcode: 0xC6, size: 2, disassembly: "ADI D8", func_symbols: "A <- A + byte", effected_flags: "Z,S,P,CY,AC".into(),
            func_ptr: |cpu, b2, _| { 
                cpu.add(b2);
            }
        });
        
        optable.insert(&Instruction { opcode: 0xC7, size: 1, disassembly: "RST 0", func_symbols: "CALL $0", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                cpu.call(0x00);
            }
        });

        optable.insert(&Instruction { opcode: 0xC8, size: 1, disassembly: "RZ", func_symbols: "if Z, RET", effected_flags: None,
            func_ptr: |cpu, _, _| { 
                if cpu.condition_codes.is_set(ConditionFlag::Zero) {
                    cpu.ret();
                }
            }
        });

        optable
    }
}

