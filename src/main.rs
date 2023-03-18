use std::collections::HashMap;
use std::env;
use std::fs::read;

const BYTE_REGISTERS: [&str; 8] = ["al", "cl", "dl", "bl", "ah", "ch", "dh", "bh"];
const WORD_REGISTERS: [&str; 8] = ["ax", "cx", "dx", "bx", "sp", "bp", "si", "di"];

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

fn parse_register_or_memory_to_or_from_register(first_byte: u8, second_byte: u8) -> String {
    let d_bit = first_byte & 0x2;
    let w_bit = first_byte & 0x1;

    let r#mod = second_byte >> 6;
    let reg = (second_byte & 0b00_111_000) >> 3;
    let rm = second_byte & 0b00_000_111;

    if r#mod != 0x3 {
        panic!("Unsupported operation. We only deal with register-to-register instructions for now. Come back later!");
    }

    let destination = if d_bit == 1 { reg } else { rm };
    let source = if d_bit == 1 { rm } else { reg };

    let source_register = if w_bit == 1 { WORD_REGISTERS[source as usize] } else { BYTE_REGISTERS[source as usize] };
    let destination_register = if w_bit == 1 { WORD_REGISTERS[destination as usize] } else { BYTE_REGISTERS[destination as usize] };

    String::from(format!("mov {destination_register}, {source_register}"))
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 || args[1].len() == 0 {
        panic!("No filename provided");
    }

    let file = read(&args[1]).expect("could not read file");

    let mut cursor = 0;
    let mut asm = String::from("bits 16\n\n");
    while cursor < file.len() {
        cursor += 2;
        let first_byte = file[cursor - 2];
        let second_byte = file[cursor - 1];

        let op = as_mov_enum(first_byte)
            .expect("Unrecognized op code. Only mov operations are supported");

        match op {
            Mov::RegisterOrMemoryToOrFromRegister => {
                asm.push_str("\n");
                asm.push_str(&parse_register_or_memory_to_or_from_register(first_byte, second_byte));
            }
            _ => {}
        }
    }

    println!("{asm}");
}
