use crate::opcodes::*;
use binrw::{BinRead, PosValue};

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
    TypeConstants,
    FunctionBlock,
}

#[derive(Debug, Clone)]
pub enum LuaSection {
    TypeConstants(TypeConstsSection),
    FunctionBlock(Box<FunctionBlock>),
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
    pub constant_type: u32,
    pub string_size: u32,

    #[br(map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string(), count = string_size)]
    pub const_string: String,
}

#[derive(BinRead, Debug, Clone, Copy)]
#[br(repr(u8))]
pub enum VarArgFlags {
    Has = 1 << 0,
    IsVar = 1 << 1,
    Unk3 = 3,
    Needs = 1 << 2,
}

#[derive(BinRead, Debug, Clone)]
#[br(big)]
pub struct FunctionBlock {
    #[br(map = |x: PosValue<()>| x.pos)]
    pub address: u64,

    pub upvalue_count: u32,     // 0x00000000 - upvalue count?
    pub param_count: u32,       // 0x00000000 - param count?
    pub vararg: VarArgFlags,    // 0x2 - is vararg?
    pub unk9: u32,              // 0x00000006 - slot count?
    pub instruction_count: u32, // 0x0000000A - instruction count
    // pub unk10: u8, // 0x5F - instruction count is 0 index, 0x4C - instruction count is 1 index???

    // #[br(count = if unk10 == 0x4C { instruction_count - 1 } else { instruction_count })]
    #[br(count = instruction_count, align_before=0x4)]
    pub instructions: Vec<LuaInstruction>,

    pub consts: BungieConstsSection,

    #[br(map = |x: u32| x == 1)]
    pub has_debug_info: bool,
    #[br(if(has_debug_info))]
    pub debug_info: DebugInfo,

    pub function_count: u32,
    // pub unk1: u32,
    #[br(count = function_count)]
    pub child_functions: Vec<ChildFunction>,
}

#[derive(BinRead, Debug, Clone)]
#[br(big)]
pub struct ChildFunction {
    #[br(map = |x: PosValue<()>| x.pos)]
    pub address: u64,

    // #[br(map = |x: u32| x == 1)]
    pub unk0: u32,
    pub upvalue_count: u32,
    pub param_count: u32,
    pub vararg: VarArgFlags,
    // pub unk9: u32, // 0x00000006 - slot count?
    pub instruction_count: u32,

    #[br(count = instruction_count, align_before=0x4)]
    pub instructions: Vec<LuaInstruction>,

    pub consts: BungieConstsSection,

    #[br(map = |x: u32| x == 1)]
    pub has_debug_info: bool,
    #[br(if(has_debug_info))]
    pub debug_info: DebugInfo,

    pub function_count: u32,
}

impl From<ChildFunction> for FunctionBlock {
    fn from(value: ChildFunction) -> Self {
        FunctionBlock {
            address: value.address,
            upvalue_count: value.upvalue_count,
            param_count: value.param_count,
            vararg: value.vararg,
            unk9: 0,
            instruction_count: value.instruction_count,
            // unk10: 0,
            instructions: value.instructions,
            consts: value.consts,
            has_debug_info: value.has_debug_info,
            debug_info: value.debug_info,
            function_count: value.function_count,
            child_functions: vec![],
        }
    }
}

impl From<FunctionBlock> for ChildFunction {
    fn from(value: FunctionBlock) -> Self {
        ChildFunction {
            address: value.address,
            unk0: 0,
            upvalue_count: value.upvalue_count,
            param_count: value.param_count,
            vararg: value.vararg,
            instruction_count: value.instruction_count,
            instructions: value.instructions,
            consts: value.consts,
            has_debug_info: value.has_debug_info,
            debug_info: value.debug_info,
            function_count: value.function_count,
        }
    }
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
    pub constant_type: u8,
    #[br(args(constant_type))]
    pub constant: BungieConstantEnum,
}

#[derive(BinRead, Debug, Clone)]
#[br(import(constant_type: u8))]
pub enum BungieConstantEnum {
    #[br(pre_assert(constant_type == 0))]
    None,
    #[br(pre_assert(constant_type == 1))]
    Bool(u8),
    #[br(pre_assert(constant_type == 2))]
    LightUserData(i64),
    #[br(pre_assert(constant_type == 3))]
    Number(f32),
    #[br(pre_assert(constant_type == 4))]
    String(BungieConstantString),
    #[br(pre_assert(constant_type == 11))]
    U64(u64),
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
