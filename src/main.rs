use std::env;
use std::fs::{read, write};

const BYTE_REGISTERS: [&str; 8] = ["al", "cl", "dl", "bl", "ah", "ch", "dh", "bh"];
const WORD_REGISTERS: [&str; 8] = ["ax", "cx", "dx", "bx", "sp", "bp", "si", "di"];
const REGISTER_ENCODINGS: [[&str; 8]; 2] = [BYTE_REGISTERS, WORD_REGISTERS];

const RM_ADDRESS_CALCULATION_ENCODINGS: [&str; 8] = [
    "[bx + si]",
    "[bx + di]",
    "[bp + si]",
    "[bp + di]",
    "[si]",
    "[di]",
    "", // to own the libs
    "[bx]",
];

fn rm_address_calculation_displaced(rm_bits: &u8, displacement: &u16) -> String {
    match rm_bits {
        0x0 => format!("[bx + si + {displacement}]"),
        0x1 => format!("[bx + di + {displacement}]"),
        0x2 => format!("[bp + si + {displacement}]"),
        0x3 => format!("[bp + di + {displacement}]"),
        0x4 => format!("[si + {displacement}]"),
        0x5 => format!("[di + {displacement}]"),
        0x6 => format!("[bp + {displacement}]"),
        0x7 => format!("[bx + {displacement}]"),
        _ => "".to_owned(),
    }
}

enum Mov {
    RegisterOrMemoryToOrFromRegister,
    ImmediateToRegisterOrMemory,
    ImmediateToRegister,
    MemoryToAccumulator,
    AccumulatorToMemory,
    RegisterOrMemoryToSegmentRegister,
    SegmentRegisterToRegisterOrMemory,
}

fn as_mov_enum(byte: u8) -> Option<Mov> {
    if byte >> 2 == 0b100010 {
        return Some(Mov::RegisterOrMemoryToOrFromRegister);
    }

    if byte >> 1 == 0b1100011 {
        return Some(Mov::ImmediateToRegisterOrMemory);
    }

    if byte >> 4 == 0b1011 {
        return Some(Mov::ImmediateToRegister);
    }

    if byte >> 1 == 0b1010000 {
        return Some(Mov::MemoryToAccumulator);
    }

    if byte >> 1 == 0b1010001 {
        return Some(Mov::AccumulatorToMemory);
    }

    if byte == 0b10001110 {
        return Some(Mov::RegisterOrMemoryToSegmentRegister);
    }

    if byte == 0b10001100 {
        return Some(Mov::SegmentRegisterToRegisterOrMemory);
    }

    None
}

fn parse_register_or_memory_to_or_from_register(bytes: &Vec<u8>, cursor: &mut usize) -> String {
    let first_byte = bytes[*cursor];
    let second_byte = bytes[*cursor + 1];
    *cursor += 2;

    let d_bit = (first_byte >> 1) & 0x1;
    let w_bit = first_byte & 0x1;

    let r#mod = second_byte >> 6;
    let register_bits = (second_byte >> 3) & 0x7;
    let rm_bits = second_byte & 0x7;

    let register = REGISTER_ENCODINGS[w_bit as usize][register_bits as usize];

    let rm = match r#mod {
        0x0 => RM_ADDRESS_CALCULATION_ENCODINGS[rm_bits as usize].to_owned(),
        0x1 => {
            let displacement = bytes[*cursor];
            *cursor += 1;
            rm_address_calculation_displaced(&rm_bits, &(displacement as u16))
        }
        0x2 => {
            let displacement = u16::from_ne_bytes([bytes[*cursor], bytes[*cursor + 1]]);
            *cursor += 2;
            rm_address_calculation_displaced(&rm_bits, &displacement)
        }
        0x3 => REGISTER_ENCODINGS[w_bit as usize][rm_bits as usize].to_owned(),
        _ => "".to_owned(),
    };

    let destination = if d_bit == 1 { register } else { &rm };
    let source = if d_bit == 1 { &rm } else { register };

    String::from(format!("mov {destination}, {source}"))
}

fn parse_immediate_to_register(bytes: &Vec<u8>, cursor: &mut usize) -> String {
    let first_byte = bytes[*cursor];
    let second_byte = bytes[*cursor + 1];
    let third_byte = bytes[*cursor + 2];

    let w_bit = (first_byte >> 3) & 0x1;
    let register_bits = first_byte & 0x07;
    let immediate: u16;
    let register: &str;

    if w_bit == 1 {
        *cursor += 3;
        immediate = u16::from_ne_bytes([second_byte, third_byte]);
        register = WORD_REGISTERS[register_bits as usize];
    } else {
        *cursor += 2;
        immediate = second_byte as u16;
        register = BYTE_REGISTERS[register_bits as usize];
    }

    format!("mov {register}, {immediate}")
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 || args[1].len() == 0 {
        panic!("No filename provided");
    }

    let file = read(&args[1]).expect("could not read input file");

    let mut cursor = 0;
    let mut asm = String::from("bits 16\n\n");
    while cursor < file.len() {
        let first_byte = file[cursor];

        let op = as_mov_enum(first_byte)
            .expect("Unrecognized op code. Only mov operations are supported");

        match op {
            Mov::RegisterOrMemoryToOrFromRegister => {
                asm.push_str("\n");
                asm.push_str(&parse_register_or_memory_to_or_from_register(
                    &file,
                    &mut cursor,
                ));
            }
            Mov::ImmediateToRegister => {
                asm.push_str("\n");
                asm.push_str(&parse_immediate_to_register(&file, &mut cursor));
            }
            _ => {
                asm.push_str("\n");
                asm.push_str("unimplemented");
            }
        }
    }

    if args.contains(&String::from("--stdio")) {
        println!("{asm}");
        return;
    }

    // maybe in the future I'll write a proper args parser 
    // and then add a -o, --output argument and only 
    // generate an output file if it's set and use its
    // value as the output file name
    write("output", &asm).expect("error trying to write to file");
}
