use std::{path::Path, fs::{OpenOptions, read_to_string}, io::Write, env, process::exit};

use pact::{prelude::*, helper::{U4, U3}};

mod error;
use error::{RimError, RimResult};

fn main() {
    let mut files = env::args();

    if files.len() != 2 {
        eprintln!(include_str!("../usage.txt"));
        return;
    }

    let input = files.next().unwrap();
    let output = files.next().unwrap();

    let instructions = match load_file(input) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("failed to parse file: {e}");
            exit(1);
        }
    };

    let bytes = instructions.into_iter()
        .map(|i| u8::from(i))
        .collect::<Vec<u8>>();

    let written = match write_file(&output, &bytes) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("failed to write to file `{output}`: {e}");
            exit(1);
        }
    };

    println!("wrote {written} bytes to {output}");
}

/// Write instruction bytes to a file, prepending the magic bytes.
/// 
/// Returns the number of bytes written.
fn write_file<F: AsRef<Path>>(file: F, data: &[u8]) -> RimResult<usize> {
    let mut file = OpenOptions::new()
        .write(true)
        .read(false)
        .truncate(true)
        .create(true)
        .open(file)?;

    let magic = MAGIC.to_be_bytes();
    file.write_all(&magic)?;
    file.write_all(data)?;

    Ok(data.len() + magic.len())
}

fn load_file<F: AsRef<Path>>(file: F) -> RimResult<Vec<Instruction>> {
    let data = read_to_string(file)?;
    let lines = data.lines();

    let mut instructions = Vec::new();
    let mut lno = 0;
    for line in lines {
        lno += 1;
        if line.chars().all(|ch| ch.is_whitespace()) {
            continue;
        }

        let parts = line.split(' ').collect::<Vec<_>>();

        let op = parts[0].to_lowercase();
        let (is_indirect, op) = if let Some(op) = op.strip_suffix('p') {
            if op == "adi" {
                return Err(RimError::InvalidInstruction(lno, "adip".to_string()));
            }

            (true, op)
        } else {
            (false, parts[0])
        };

        let opcode = match op {
            "adi" => Opcode::Adi,
            "add" => Opcode::Add,
            "sub" => Opcode::Sub,
            "jne" => Opcode::Jne,
            "jg" => Opcode::Jg,
            "jl" => Opcode::Jl,
            "ioi" => Opcode::Ioi,
            "ior" => Opcode::Ior,

            "hlt" => {
                instructions.push(Instruction(Opcode::Ioi, InstructionData::Io { device: Device::Cpu, function: U3::from(0) }));
                continue;
            }

            bad => return Err(RimError::InvalidInstruction(lno, bad.to_string()))
        };

        match opcode {
            Opcode::Adi => {
                if parts.len() < 2 {
                    return Err(RimError::LineTooShort(lno));
                } else if parts.len() > 2 {
                    return Err(RimError::LineTooLong(lno));
                }

                let val = parts[1].parse::<u8>().map_err(|_| RimError::InvalidInteger(lno, parts[1].to_string()))?;
                if val > 31 {
                    return Err(RimError::IntegerTooLarge(lno, val, 31));
                }

                instructions.push(Instruction(opcode, InstructionData::Imm(val)));
            }

            Opcode::Add
            | Opcode::Sub => {
                if parts.len() < 3 {
                    return Err(RimError::LineTooShort(lno));
                } else if parts.len() > 3 {
                    return Err(RimError::LineTooLong(lno));
                }

                let src = match parts[1].to_lowercase().as_str() {
                    "ra" => Register::Ra,
                    "rb" => Register::Rb,
                    "rc" => Register::Rc,
                    "rd" => Register::Rd,

                    src => return Err(RimError::InvalidRegister(lno, src.to_string()))
                };

                let dest = match parts[2].to_lowercase().as_str() {
                    "ra" => Register::Ra,
                    "rb" => Register::Rb,
                    "rc" => Register::Rc,
                    "rd" => Register::Rd,

                    dest => return Err(RimError::InvalidRegister(lno, dest.to_string()))
                };

                instructions.push(Instruction(opcode, InstructionData::Reg { is_id: is_indirect, src, dest }));
            }

            Opcode::Jne
            | Opcode::Jg
            | Opcode::Jl => {
                if parts.len() < 2 {
                    return Err(RimError::LineTooShort(lno));
                } else if parts.len() > 2 {
                    return Err(RimError::LineTooLong(lno));
                }

                let addr = parts[1].parse::<u8>().map_err(|_| RimError::InvalidInteger(lno, parts[1].to_string()))?;
                if addr > 16 {
                    return Err(RimError::IntegerTooLarge(lno, addr, 16));
                }

                instructions.push(Instruction(opcode, InstructionData::Mem { is_ptr: is_indirect, addr: U4::from(addr) }));
            }

            Opcode::Ioi
            | Opcode::Ior => {
                if parts.len() < 3 {
                    return Err(RimError::LineTooShort(lno));
                } else if parts.len() > 3 {
                    return Err(RimError::LineTooLong(lno));
                }

                let device = match parts[1].to_lowercase().as_str() {
                    "cpu" => Device::Cpu,
                    "kbd" => Device::Kbd,
                    "scr" => Device::Scr,
                    "mth" => Device::Mth,
                    dev => return Err(RimError::InvalidDevice(lno, dev.to_string()))
                };

                let function = parts[2].parse::<u8>().map_err(|_| RimError::InvalidInteger(lno, parts[2].to_string()))?;
                if function > 7 {
                    return Err(RimError::IntegerTooLarge(lno, function, 7));
                }

                instructions.push(Instruction(opcode, InstructionData::Io { device, function: U3::from(function) }));
            }
        }
    }

    Ok(instructions)
}
