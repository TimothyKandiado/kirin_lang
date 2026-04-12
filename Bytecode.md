# Bytecode

All Instructions are 32 bits

## Format A (R type, 3 registers)

Opcode (7) | dst (6) | src (6) | src 2 (6) | flags (7)

### Arithmetic

- add_i64
- sub_i64
- mul_i64
- div_i64
- mod_i64
- pow_i64


### Type conversion

- i64_to_f64
- f64_to_i64

### Comparisons
- cmp_eq_i64
- cmp_ne_i64
- cmp_lt_i64
- cmp_le_i64
- cmp_gt_i64
- cmp_ge_i64

- cmp_eq_f64
- cmp_ne_f64
- cmp_lt_f64
- cmp_le_f64
- cmp_gt_f64
- cmp_ge_f64

### Boolean operations
- and_bool
- or_bool
- not_bool


## Format B (R + immediate)

Opcode (7) | dst (6) | src (6) | const_index(13)

- BOX


## Format C (Constant load)

Opcode (7bits) | dst (6bits) | const_index(19)

- CONST_I64
- CONST_I64_IMM
- CONST_F64
- CONST_TRUE
- CONST_FALSE
- CONST_STR


## Format D (Branch)

Opcode (7bits) | reg (6) | offset (19)

### Unconditional jump
- JMP offset

### Conditional jump
- BR_TRUE reg offset

### Function control
- RET reg
- RET_VOID

## Format E (Call)

Opcode (7bits) | ret_reg (6bits) | arg_reg (6bits) | func_index (13)

### Function call

- CALL ret_reg arg_reg func_index