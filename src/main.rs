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
    "[bp]",
    "[bx]",
];

fn rm_address_calculation_displaced(rm_bits: &u8, displacement: &i16) -> String {
    let sign = if displacement > &1 { "+" } else { "-" };
    let abs_displacement = displacement.abs();
    match rm_bits {
        0x0 => format!("[bx + si {sign} {abs_displacement}]"),
        0x1 => format!("[bx + di {sign} {abs_displacement}]"),
        0x2 => format!("[bp + si {sign} {abs_displacement}]"),
        0x3 => format!("[bp + di {sign} {abs_displacement}]"),
        0x4 => format!("[si {sign} {abs_displacement}]"),
        0x5 => format!("[di {sign} {abs_displacement}]"),
        0x6 => format!("[bp {sign} {abs_displacement}]"),
        0x7 => format!("[bx {sign} {abs_displacement}]"),
        _ => "".to_owned(),
    }
}

#[derive(Debug)]
enum Opcode {
    MovRegisterOrMemoryToOrFromRegister,
    MovImmediateToRegisterOrMemory,
    MovImmediateToRegister,
    MovMemoryToAccumulator,
    MovAccumulatorToMemory,
    MovRegisterOrMemoryToSegmentRegister,
    MovSegmentRegisterToRegisterOrMemory,
    AddRegisterOrMemoryWithRegisterToEither,
    AddImmediateToRegisterOrMemory,
    AddImmediateToAccumulator,
    SubRegisterOrMemoryWithRegisterToEither,
    SubImmediateToRegisterOrMemory,
    SubImmediateToAccumulator,
    CmpRegisterOrMemoryAndRegister,
    CmpImmediateWithRegisterOrMemory,
    CmpImmediateWithAccumulator,
}

fn as_opcode_enum(bytes: [u8; 2]) -> Option<Opcode> {
    if bytes[0] >> 2 == 0b100010 {
        return Some(Opcode::MovRegisterOrMemoryToOrFromRegister);
    }

    if bytes[0] >> 1 == 0b1100011 {
        return Some(Opcode::MovImmediateToRegisterOrMemory);
    }

    if bytes[0] >> 4 == 0b1011 {
        return Some(Opcode::MovImmediateToRegister);
    }

    if bytes[0] >> 1 == 0b1010000 {
        return Some(Opcode::MovMemoryToAccumulator);
    }

    if bytes[0] >> 1 == 0b1010001 {
        return Some(Opcode::MovAccumulatorToMemory);
    }

    if bytes[0] == 0b10001110 {
        return Some(Opcode::MovRegisterOrMemoryToSegmentRegister);
    }

    if bytes[0] == 0b10001100 {
        return Some(Opcode::MovSegmentRegisterToRegisterOrMemory);
    }

    if bytes[0] >> 2 == 0b000000 {
        return Some(Opcode::AddRegisterOrMemoryWithRegisterToEither);
    }

    if bytes[0] >> 2 == 0b100000 {
        let reg = bytes[1] >> 3 & 0x7;
        if reg == 0b101 {
            return Some(Opcode::SubImmediateToRegisterOrMemory);
        } else if reg == 0b111 {
            return Some(Opcode::CmpImmediateWithRegisterOrMemory);
        } else if reg == 0b0 {
            return Some(Opcode::AddImmediateToRegisterOrMemory);
        }
    }

    if bytes[0] >> 1 == 0b0000010 {
        return Some(Opcode::AddImmediateToAccumulator);
    }

    if bytes[0] >> 2 == 0b001010 {
        return Some(Opcode::SubRegisterOrMemoryWithRegisterToEither);
    }

    if bytes[0] >> 2 == 0b100000 {
        return Some(Opcode::SubImmediateToRegisterOrMemory);
    }

    if bytes[0] >> 1 == 0b0010110 {
        return Some(Opcode::SubImmediateToAccumulator);
    }

    if bytes[0] >> 2 == 0b001110 {
        return Some(Opcode::CmpRegisterOrMemoryAndRegister);
    }

    if bytes[0] >> 1 == 0b0011110 {
        return Some(Opcode::CmpImmediateWithAccumulator);
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
        0x0 => {
            if rm_bits != 0x6 {
                RM_ADDRESS_CALCULATION_ENCODINGS[rm_bits as usize].to_owned()
            } else {
                if w_bit == 0 {
                    let disp_lo = bytes[*cursor];
                    *cursor += 1;

                    let is_displacement_signed = ((disp_lo >> 7) & 0x1) == 1;
                    let displacement = if is_displacement_signed {
                        (disp_lo.wrapping_neg() as i16) * -1
                    } else {
                        disp_lo as i16
                    };

                    format!("[{displacement}]")
                } else {
                    let disp_lo = bytes[*cursor];
                    let disp_hi = bytes[*cursor + 1];
                    *cursor += 2;

                    let displacement = i16::from_ne_bytes([disp_lo, disp_hi]);
                    format!("[{displacement}]")
                }
            }
        }
        0x1 => {
            let is_displacement_signed = ((bytes[*cursor] >> 7) & 0x1) == 1;
            let displacement = if is_displacement_signed {
                (bytes[*cursor].wrapping_neg() as i16) * -1
            } else {
                bytes[*cursor] as i16
            };
            *cursor += 1;
            rm_address_calculation_displaced(&rm_bits, &(displacement as i16))
        }
        0x2 => {
            let displacement = i16::from_ne_bytes([bytes[*cursor], bytes[*cursor + 1]]);
            *cursor += 2;
            rm_address_calculation_displaced(&rm_bits, &displacement)
        }
        0x3 => REGISTER_ENCODINGS[w_bit as usize][rm_bits as usize].to_owned(),
        _ => "".to_owned(),
    };

    let destination = if d_bit == 1 { register } else { &rm };
    let source = if d_bit == 1 { &rm } else { register };

    let operation = if first_byte >> 2 == 0b10010 {
        "mov"
    } else if first_byte >> 2 == 0b0 {
        "add"
    } else if first_byte >> 2 == 0b001010 {
        "sub"
    } else if first_byte >> 2 == 0b001110 {
        "cmp"
    } else {
        ""
    };
    String::from(format!("{operation} {destination}, {source}"))
}

fn parse_immediate_to_register(bytes: &Vec<u8>, cursor: &mut usize) -> String {
    let first_byte = bytes[*cursor];
    let data_lo = bytes[*cursor + 1];
    *cursor += 2;

    let w_bit = (first_byte >> 3) & 0x1;
    let register_bits = first_byte & 0x07;
    let immediate: u16;
    let register: &str;

    if w_bit == 1 {
        let data_hi = bytes[*cursor];
        *cursor += 1;
        immediate = u16::from_ne_bytes([data_lo, data_hi]);
        register = WORD_REGISTERS[register_bits as usize];
    } else {
        immediate = data_lo as u16;
        register = BYTE_REGISTERS[register_bits as usize];
    }

    format!("mov {register}, {immediate}")
}

fn parse_immediate_to_register_or_memory(bytes: &Vec<u8>, cursor: &mut usize) -> String {
    let first_byte = bytes[*cursor];
    let second_byte = bytes[*cursor + 1];
    *cursor += 2;

    let w_bit = first_byte & 0x1;
    let r#mod = (second_byte >> 6) & 0x03;
    let rm_bits = second_byte & 0x07;
    let immediate: u16;

    let rm = match r#mod {
        0x0 => {
            if rm_bits != 0x6 {
                RM_ADDRESS_CALCULATION_ENCODINGS[rm_bits as usize].to_owned()
            } else {
                if w_bit == 0 {
                    let disp_lo = bytes[*cursor];
                    *cursor += 1;

                    let is_displacement_signed = ((disp_lo >> 7) & 0x1) == 1;
                    let displacement = if is_displacement_signed {
                        (disp_lo.wrapping_neg() as i16) * -1
                    } else {
                        disp_lo as i16
                    };

                    format!("[{displacement}]")
                } else {
                    let disp_lo = bytes[*cursor];
                    let disp_hi = bytes[*cursor + 1];
                    *cursor += 2;

                    let displacement = i16::from_ne_bytes([disp_lo, disp_hi]);
                    format!("[{displacement}]")
                }
            }
        }
        0x1 => {
            let disp_lo = bytes[*cursor];
            *cursor += 1;

            let is_displacement_signed = ((disp_lo >> 7) & 0x1) == 1;
            let displacement = if is_displacement_signed {
                (disp_lo.wrapping_neg() as i16) * -1
            } else {
                disp_lo as i16
            };
            rm_address_calculation_displaced(&rm_bits, &(displacement as i16))
        }
        0x2 => {
            let disp_lo = bytes[*cursor];
            let disp_hi = bytes[*cursor + 1];
            *cursor += 2;

            let displacement = i16::from_ne_bytes([disp_lo, disp_hi]);
            rm_address_calculation_displaced(&rm_bits, &displacement)
        }
        0x3 => {
            if w_bit == 1 {
                WORD_REGISTERS[rm_bits as usize].to_owned()
            } else {
                BYTE_REGISTERS[rm_bits as usize].to_owned()
            }
        }
        _ => panic!(),
    };

    let register_bits = (second_byte >> 3) & 0x7;
    let operation = if first_byte >> 2 == 0b100010 {
        "mov"
    } else if first_byte >> 2 == 0b100000 && register_bits == 0b0 {
        "add"
    } else if first_byte >> 2 == 0b100000 && register_bits == 0b101 {
        "sub"
    } else if first_byte >> 2 == 0b100000 && register_bits == 0b111 {
        "cmp"
    } else {
        ""
    };

    let size = if w_bit == 1 { "word" } else { "byte" };
    if operation == "mov" {
        if w_bit == 1 {
            let data_lo = bytes[*cursor];
            let data_hi = bytes[*cursor + 1];
            *cursor += 2;

            immediate = u16::from_ne_bytes([data_lo, data_hi]);
        } else {
            let data_lo = bytes[*cursor];
            *cursor += 1;

            immediate = data_lo as u16;
        }
    } else {
        let s_bit = (first_byte >> 1) & 0x1;
        if w_bit == 1 && s_bit == 0 {
            let data_lo = bytes[*cursor];
            let data_hi = bytes[*cursor + 1];
            *cursor += 2;

            immediate = u16::from_ne_bytes([data_lo, data_hi]);
        } else {
            let data_lo = bytes[*cursor];
            *cursor += 1;

            immediate = data_lo as u16;
        }
    }

    if first_byte >> 2 == 0b100010 {
        format!("mov {rm}, {size} {immediate}")
    } else if first_byte >> 2 == 0b100000 && register_bits == 0b0 {
        format!("add {size} {rm}, {immediate}")
    } else if first_byte >> 2 == 0b100000 && register_bits == 0b101 {
        format!("sub {size} {rm}, {immediate}")
    } else if first_byte >> 2 == 0b100000 && register_bits == 0b111 {
        format!("cmp {size} {rm}, {immediate}")
    } else {
        "".to_owned()
    }
}

fn parse_memory_to_accumulator(bytes: &Vec<u8>, cursor: &mut usize) -> String {
    let first_byte = bytes[*cursor];
    *cursor += 1;

    let w_bit = first_byte & 0x1;

    if w_bit == 1 {
        let addr_lo = bytes[*cursor];
        let addr_hi = bytes[*cursor + 1];
        *cursor += 2;

        let address = u16::from_ne_bytes([addr_lo, addr_hi]);
        format!("mov ax, [{address}]")
    } else {
        let addr_lo = bytes[*cursor];
        *cursor += 1;

        format!("mov al, [{addr_lo}]")
    }
}

fn parse_accumulator_to_memory(bytes: &Vec<u8>, cursor: &mut usize) -> String {
    let first_byte = bytes[*cursor];
    *cursor += 1;

    let w_bit = first_byte & 0x1;

    if w_bit == 1 {
        let addr_lo = bytes[*cursor];
        let addr_hi = bytes[*cursor + 1];
        *cursor += 2;

        let address = u16::from_ne_bytes([addr_lo, addr_hi]);
        format!("mov [{address}], ax")
    } else {
        let addr_lo = bytes[*cursor];
        *cursor += 1;

        let address = addr_lo;
        format!("mov [{address}], al")
    }
}

fn parse_immediate_to_accumulator(bytes: &Vec<u8>, cursor: &mut usize) -> String {
    let first_byte = bytes[*cursor];
    *cursor += 1;

    let w_bit = first_byte & 0x1;

    let operation = if first_byte >> 1 == 0b0010110 {
        "sub"
    } else if first_byte >> 1 == 0b0000010 {
        "add"
    } else if first_byte >> 1 == 0b0011110 {
        "cmp"
    } else {
        ""
    };

    if w_bit == 1 {
        let data = u16::from_ne_bytes([bytes[*cursor], bytes[*cursor + 1]]);
        *cursor += 2;
        format!("{operation} ax, {data}")
    } else {
        let data = bytes[*cursor] as i8;
        *cursor += 1;
        format!("{operation} al, {data}")
    }
}

fn parse_bin(bin: Vec<u8>) -> String {
    let mut cursor = 0;
    let mut asm = String::from("bits 16\n\n");

    while cursor < bin.len() {
        let first_two_bytes = [bin[cursor], bin[cursor + 1]];

        let op = as_opcode_enum(first_two_bytes)
            .expect(format!("Unrecognized opcode. {:0>8b}", first_two_bytes[0]).as_str());

        match op {
            Opcode::MovRegisterOrMemoryToOrFromRegister
            | Opcode::AddRegisterOrMemoryWithRegisterToEither
            | Opcode::SubRegisterOrMemoryWithRegisterToEither
            | Opcode::CmpRegisterOrMemoryAndRegister => {
                asm.push_str("\n");
                asm.push_str(&parse_register_or_memory_to_or_from_register(
                    &bin,
                    &mut cursor,
                ));
            }
            Opcode::MovImmediateToRegister => {
                asm.push_str("\n");
                asm.push_str(&parse_immediate_to_register(&bin, &mut cursor));
            }
            Opcode::MovImmediateToRegisterOrMemory
            | Opcode::AddImmediateToRegisterOrMemory
            | Opcode::SubImmediateToRegisterOrMemory
            | Opcode::CmpImmediateWithRegisterOrMemory => {
                asm.push_str("\n");
                asm.push_str(&parse_immediate_to_register_or_memory(&bin, &mut cursor));
            }
            Opcode::MovMemoryToAccumulator => {
                asm.push_str("\n");
                asm.push_str(&parse_memory_to_accumulator(&bin, &mut cursor));
            }
            Opcode::MovAccumulatorToMemory => {
                asm.push_str("\n");
                asm.push_str(&parse_accumulator_to_memory(&bin, &mut cursor));
            }
            Opcode::AddImmediateToAccumulator
            | Opcode::SubImmediateToAccumulator
            | Opcode::CmpImmediateWithAccumulator => {
                asm.push_str("\n");
                asm.push_str(&parse_immediate_to_accumulator(&bin, &mut cursor));
            }
            _ => {
                panic!("found unimplemented op")
            }
        }
    }

    asm
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 || args[1].len() == 0 {
        panic!("No filename provided");
    }

    let file = read(&args[1]).expect("could not read input file");

    let asm = parse_bin(file);

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

#[cfg(test)]
mod tests {
    use std::num::ParseIntError;

    use super::*;

    pub fn hex_to_bin(s: &str) -> Result<Vec<u8>, ParseIntError> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
            .collect()
    }

    #[test]
    fn add_positive_immediate_to_accumulator() {
        assert_eq!(
            parse_bin(hex_to_bin("05e803").unwrap()),
            "bits 16\n\n\nadd ax, 1000"
        );
    }

    #[test]
    fn add_negative_immediate_to_accumulator() {
        assert_eq!(
            parse_bin(hex_to_bin("04e2").unwrap()),
            "bits 16\n\n\nadd al, -30"
        );
    }

    #[test]
    fn add_immediate_to_displaced_memory() {
        assert_eq!(
            parse_bin(hex_to_bin("8382e8031d").unwrap()),
            "bits 16\n\n\nadd word [bp + si + 1000], 29"
        );
    }

    #[test]
    fn sub_positive_immediate_from_memory() {
        assert_eq!(
            parse_bin(hex_to_bin("802f22").unwrap()),
            "bits 16\n\n\nsub byte [bx], 34"
        );
    }

    #[test]
    fn sub_immediate_from_accumulator() {
        assert_eq!(
            parse_bin(hex_to_bin("2c09").unwrap()),
            "bits 16\n\n\nsub al, 9"
        );
    }

    #[test]
    fn comp_register_and_memory() {
        assert_eq!(
            parse_bin(hex_to_bin("3b18").unwrap()),
            "bits 16\n\n\ncmp bx, [bx + si]"
        );
    }

    #[test]
    fn comp_immediate_with_register() {
        assert_eq!(
            parse_bin(hex_to_bin("83fe02").unwrap()),
            "bits 16\n\n\ncmp word si, 2"
        );
    }

    #[test]
    fn comp_immediate_with_accumulator() {
        assert_eq!(
            parse_bin(hex_to_bin("3de803").unwrap()),
            "bits 16\n\n\ncmp ax, 1000"
        )
    }
}
