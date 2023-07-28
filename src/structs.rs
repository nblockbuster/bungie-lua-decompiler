use binrw::{BinRead};
use crate::opcodes::*;

#[derive(BinRead, Debug)]
#[br(repr = u8)]
pub enum LuaEndian {
    Big,
    Little,
}

#[derive(BinRead, Debug)]
#[br(repr = u8)]
pub enum LuaNumberType {
    Float,
    Integer,
}

#[derive(BinRead, Debug)]
#[br(big, magic = b"\x1bLua")]
pub struct LuaHeader {
    pub version: u8,
    pub format: u8,
    pub endianness: LuaEndian,
    pub int_size: u8,
    pub size_t: u8,
    pub instruction_size: u8,
    pub number_size: u8,
    pub number_type: LuaNumberType,
    pub integral_flag: u8,
    pub unk: u8,
}

#[derive(Debug)]
pub enum LuaSectionType {
    TypeConstants, // type defs
    Unk1, // vec of unk32s, after a few u32s and a u8
    BungieConstants,
    Debug,
}

#[derive(Debug, Clone)]
pub enum LuaSection {
    TypeConstants(TypeConstsSection),
    Unk1(UnkSection1),
    BungieConstants(BungieConstsSection),
    Debug(DebugSection),
}

#[derive(BinRead, Debug, Clone)]
#[br(big)]
pub struct TypeConstsSection {
    pub constants_amount: u32,
    #[br(count = constants_amount)]
    pub constants: Vec<LuaConstant>,
}

#[derive(BinRead, Debug, Clone)]
#[br(big)]
pub struct LuaConstant {
    pub constant_type: u32, // TODO: enum generated from the first consts field?
    pub string_size: u32,

    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string(), count = string_size)]
    pub const_string: String,
}

#[derive(BinRead, Debug, Clone)]
#[br(big)]
pub struct UnkSection1 {
    // #[br(pad_before = 0x1)]
    pub unk0: u32, // 0x00000000 - upvalue count?
    pub unk4: u32, // 0x00000000 - param count?
    pub unk8: u8, // 0x2 - is vararg?
    pub unk9: u32, // 0x00000006 - slot count?
    pub unk_count: u32, // 0x0000000A - instruction count
    pub unk10: u8, // 0x5F - unk

    #[br(count = unk_count)]
    pub instructions: Vec<LuaInstruction>,
}

#[derive(BinRead, Debug, Clone)]
#[br(big)]
pub struct LuaInstruction {
    pub raw: u32,
    // opcode is raw >> 25 cast to OpCode enum
    #[br(ignore)]
    pub opcode: OpCode,
    // opmodes is opcode's position in the opmode
    #[br(ignore)]
    pub opmodes: OpModes,
    #[brw(ignore)]
    pub args: Vec<OpArg>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OpArg {
    pub mode: OpArgMode,
    pub value: u32,
}

#[derive(BinRead, Debug, Clone)]
#[br(big)]
pub struct BungieConstsSection {
    // #[br(pad_before = 0x1)]
    pub constants_amount: u32,
    #[br(count = constants_amount)]
    pub constants: Vec<BungieConstant>,
}

#[derive(BinRead, Debug, Clone)]
#[br(big)]
pub struct BungieConstant {
    // TODO: can be sizes that arent u8?

    pub constant_type: u8, // TODO: enum?

    #[br(if(constant_type == 1), map = |x: u8| x == 1)]
    pub constant_bool: bool,
    #[br(if(constant_type == 2))]
    pub constant_lightuserdata: i64,
    #[br(if(constant_type == 3))]
    pub constant_number: f32,
    #[br(if(constant_type == 4))]
    pub constant_string: BungieConstantString,
    #[br(if(constant_type == 11))]
    pub constant_u64: u64,
}

#[derive(BinRead, Debug, Default, Clone)]
#[br(big)]
pub struct BungieConstantString {
    pub string_size: u32,

    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string(), count = string_size)]
    pub const_string: String,
}

#[derive(BinRead, Debug, Default, Clone)]
#[br(big)]
pub struct DebugSection {
    #[br(map = |x: u32| x == 1)]
    pub has_debug_info: bool,
    #[br(if(has_debug_info))]
    pub debug_info: DebugInfo,
}

#[derive(BinRead, Debug, Default, Clone)]
#[br(big)]
pub struct DebugInfo {
    pub line_count: u32,
    pub locals_count: u32,
    pub upvalue_count_2: u32,
    pub line_begin: u32,
    pub line_end: u32,

    pub path_string_size: u32,
    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string(), count = path_string_size)]
    pub path: String,

    pub function_string_size: u32,
    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string(), count = function_string_size)]
    pub function_name: String,

    #[br(count = line_count)]
    pub lines: Vec<u32>,

    #[br(count = locals_count)]
    pub locals: Vec<DebugLocal>,

    #[br(count = upvalue_count_2)]
    pub upvalues: Vec<DebugUpvalue>,
}

#[derive(BinRead, Debug, Default, Clone)]
#[br(big)]
pub struct DebugLocal {
    pub string_size: u32,
    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string(), count = string_size)]
    pub local_name: String,
    pub start: i32,
    pub end: i32,
}

#[derive(BinRead, Debug, Default, Clone)]
#[br(big)]
pub struct DebugUpvalue {
    pub string_size: u32,
    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string(), count = string_size)]
    pub string: String,
}