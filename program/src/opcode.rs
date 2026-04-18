pub const OP_NO_OP: u8 = 0x00;

pub const OP_CONST_I64_IMM: u8 = 0x01;
pub const OP_CONST_I64: u8 = 0x02;
pub const OP_CONST_F64: u8 = 0x03;
pub const OP_CONST_TRUE: u8 = 0x04;
pub const OP_CONST_FALSE: u8 = 0x05;
pub const OP_CONST_STR: u8 = 0x06;

pub const OP_MOVE: u8 = 0x07;
pub const OP_SWAP: u8 = 0x08;

pub const OP_ADD_I64: u8 = 0x09;
pub const OP_SUB_I64: u8 = 0x0A;
pub const OP_MUL_I64: u8 = 0x0B;
pub const OP_DIV_I64: u8 = 0x0C;
pub const OP_MOD_I64: u8 = 0x0D;
pub const OP_POW_I64: u8 = 0x0E;
pub const OP_NEG_I64: u8 = 0x0F;

pub const OP_CMP_LT_I64: u8 = 0x10;
pub const OP_CMP_LE_I64: u8 = 0x11;

pub const OP_CMP_EQ: u8 = 0x12;

pub const OP_NOT: u8 = 0x13;
pub const OP_AND: u8 = 0x14;
pub const OP_OR: u8 = 0x15;

pub const OP_BR_FALSE: u8 = 0x16;
pub const OP_JUMP: u8 = 0x17;

pub const OP_CALL: u8 = 0x18;
pub const OP_INVOKE: u8 = 0x19;
pub const OP_RET: u8 = 0x1A;
pub const OP_RET_VOID: u8 = 0x1B;
pub const OP_HALT: u8 = 0x1C;

pub fn opcode_name(op: u8) -> &'static str {
    match op {
        OP_NO_OP => "OP_NO_OP",

        OP_CONST_I64_IMM => "OP_CONST_I64_IMM",
        OP_CONST_I64 => "OP_CONST_I64",
        OP_CONST_F64 => "OP_CONST_F64",
        OP_CONST_TRUE => "OP_CONST_TRUE",
        OP_CONST_FALSE => "OP_CONST_FALSE",
        OP_CONST_STR => "OP_CONST_STR",

        OP_MOVE => "OP_MOVE",
        OP_SWAP => "OP_SWAP",

        OP_ADD_I64 => "OP_ADD_I64",
        OP_SUB_I64 => "OP_SUB_I64",
        OP_MUL_I64 => "OP_MUL_I64",
        OP_DIV_I64 => "OP_DIV_I64",
        OP_MOD_I64 => "OP_MOD_I64",
        OP_POW_I64 => "OP_POW_I64",
        OP_NEG_I64 => "OP_NEG_I64",

        OP_CMP_LT_I64 => "OP_CMP_LT_I64",
        OP_CMP_LE_I64 => "OP_CMP_LE_I64",

        OP_CMP_EQ => "OP_CMP_EQ",

        OP_NOT => "OP_NOT",
        OP_AND => "OP_AND",
        OP_OR => "OP_OR",

        OP_BR_FALSE => "OP_BR_FALSE",
        OP_JUMP => "OP_JUMP",

        OP_CALL => "OP_CALL",
        OP_INVOKE => "OP_INVOKE",
        OP_RET => "OP_RET",
        OP_RET_VOID => "OP_RET_VOID",
        OP_HALT => "OP_HALT",

        _ => "UNKNOWN_OPCODE",
    }
}
