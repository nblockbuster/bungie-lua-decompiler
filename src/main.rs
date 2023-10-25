mod opcodes;
mod structs;

use crate::opcodes::*;
use crate::structs::*;
use binrw::BinReaderExt;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let mut input: &Path = Path::new("");
    if args.len() < 2 {
        println!("No input file specified!");
        println!("Usage: {} <input file>", args[0]);
        return;
    }
    if args.len() > 1 {
        println!("Reading from file {}", args[1]);
        input = Path::new(&args[1]);
    }
    let mut file = File::open(input).unwrap();
    let mut file_data = Vec::new();
    file.read_to_end(&mut file_data).unwrap();
    let mut reader = Cursor::new(file_data);
    let header = parse_lua_header(&mut reader);
    println!("Header: {:#?}", header);
    if header.version != 0x51 {
        println!("No known Bungie or Encounter Script that uses Lua 5.1");
        return;
    }
    if header.format != 0xE {
        println!("No known Bungie or Encounter Script that does not have format 0xE");
        return;
    }

    let mut section = parse_lua_section(&mut reader, LuaSectionType::TypeConstants);
    println!("Section: {:#?}", section);

    // println!("reader pos: {}", reader.position());
    section = parse_lua_section(&mut reader, LuaSectionType::FunctionBlock);
    // println!("reader pos: {}", reader.position());
    section = parse_instructions(section);
    println!("Section: {:#?}", section);

    // println!("reader pos: {}", reader.position());
}

fn parse_instructions(section: LuaSection) -> LuaSection {
    let mut unk1sec = match section {
        LuaSection::FunctionBlock(s) => s,
        _ => panic!("Expected FunctionBlock"),
    };
    let instructions: Vec<LuaInstruction> = unk1sec.instructions;

    let mut new_instructions: Vec<LuaInstruction> = Vec::new();

    for mut instruction in instructions.clone() {
        instruction.opcode = OpCode::from((instruction.raw >> 25) as u8);
        instruction.opmodes = OP_MODES[instruction.opcode as usize];
        let opmodes = instruction.opmodes;
        let mode = match instruction.opmodes.arg_mode_a {
            OpArgModeA::UNUSED => OpArgMode::NUMBER,
            OpArgModeA::REG => OpArgMode::REG,
        };
        let value = instruction.raw & 0xff;
        instruction.args.push(OpArg { mode, value });

        if opmodes.mode == OpMode::iABC {
            if opmodes.arg_mode_b != OpArgModeBC::UNUSED {
                let mut mode: OpArgMode = OpArgMode::NUMBER;
                let mut value: u32 = 0;
                match opmodes.arg_mode_b {
                    OpArgModeBC::NUMBER => {
                        mode = OpArgMode::NUMBER;
                        value = instruction.raw >> 17 & 0xff;
                    }
                    OpArgModeBC::OFFSET => {
                        mode = OpArgMode::NUMBER;
                        value = instruction.raw >> 17 & 0x1ff;
                    }
                    OpArgModeBC::REG => {
                        mode = OpArgMode::REG;
                        value = (instruction.raw >> 17) & 0xff;
                    }
                    OpArgModeBC::REG_OR_CONST => {
                        value = (instruction.raw >> 17) & 0x1ff;
                        if value < 0x100 {
                            mode = OpArgMode::REG;
                        } else {
                            mode = OpArgMode::CONST;
                            value &= 0xff;
                        }
                    }
                    OpArgModeBC::CONST => {
                        mode = OpArgMode::CONST;
                        value = (instruction.raw >> 17) & 0xff;
                    }
                    _ => {}
                }
                instruction.args.push(OpArg { mode, value });
            }

            if opmodes.arg_mode_c != OpArgModeBC::UNUSED {
                let mut mode: OpArgMode = OpArgMode::NUMBER;
                let mut value: u32 = 0;
                match opmodes.arg_mode_c {
                    OpArgModeBC::NUMBER => {
                        mode = OpArgMode::NUMBER;
                        value = instruction.raw >> 8 & 0xff;
                    }
                    OpArgModeBC::OFFSET => {
                        mode = OpArgMode::NUMBER;
                        value = instruction.raw >> 8 & 0x1ff;
                    }
                    OpArgModeBC::REG => {
                        mode = OpArgMode::REG;
                        value = (instruction.raw >> 8) & 0xff;
                    }
                    OpArgModeBC::REG_OR_CONST => {
                        value = (instruction.raw >> 8) & 0x1ff;
                        if value < 0x100 {
                            mode = OpArgMode::REG;
                        } else {
                            mode = OpArgMode::CONST;
                            value &= 0xff;
                        }
                    }
                    OpArgModeBC::CONST => {
                        mode = OpArgMode::CONST;
                        value = (instruction.raw >> 8) & 0xff;
                    }
                    _ => {}
                }
                instruction.args.push(OpArg { mode, value });
            }
        } else if opmodes.arg_mode_b != OpArgModeBC::UNUSED {
            let mut value = instruction.raw >> 8 & 0x1ffff;
            if opmodes.mode == OpMode::iAsBx {
                value -= 0xffff;
            }
            let mode = match opmodes.arg_mode_b {
                OpArgModeBC::OFFSET => OpArgMode::NUMBER,
                OpArgModeBC::CONST => OpArgMode::CONST,
                _ => OpArgMode::NUMBER,
            };
            instruction.args.push(OpArg { mode, value });
        }

        new_instructions.push(instruction);
    }

    unk1sec.instructions = new_instructions;

    for child in unk1sec.child_functions.iter_mut() {
        let func_block: FunctionBlock = child.clone().into();
        let a = parse_instructions(LuaSection::FunctionBlock(Box::new(func_block)));
        if let LuaSection::FunctionBlock(b) = a {
            *child = FunctionBlock::into(*b);
        }
    }

    LuaSection::FunctionBlock(unk1sec)
}

fn parse_lua_header(reader: &mut Cursor<Vec<u8>>) -> LuaHeader {
    let header: LuaHeader = reader.read_be().unwrap();
    header
}

fn parse_lua_section(reader: &mut Cursor<Vec<u8>>, sec_type: LuaSectionType) -> LuaSection {
    match sec_type {
        LuaSectionType::TypeConstants => {
            let section: TypeConstsSection = reader.read_be().unwrap();
            LuaSection::TypeConstants(section)
        }
        LuaSectionType::FunctionBlock => {
            let section: FunctionBlock = reader.read_be().unwrap();
            LuaSection::FunctionBlock(Box::new(section))
        }
    }
}
