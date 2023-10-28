// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// the fundanmental of XiaoXuan Core VM Assembly s-expression
// ----------------------------------------------------------
//
// 1. the assembly text is in s-expression format, and the text consists of
//    one or more nodes.
// 2. each node consists of a pair of parentheses, a node name and one or more
//    arguments. e.g.
//
//    `(node_name param0 param1 param2)`
//
// 3. the parameter values can be symbols, identifiers, numbers and strings.
//    parameter values can also be nodes, so the assembly text looks like a
//    tree structure. e.g.
//
//    ```clojure
//    (fn $id (param $lhs i32) (param $rhs i32) (result i32)
//        (code
//            (i32.add
//                (local.load32 $lhs) (local.load32 $rhs)
//            )
//        )
//    )
//    ```
//
// 4. the parameters have a fixed order and the positions of the parameters
//    cannot be changed.
//
//    `(local $sum i32)` =/= `(local i32 $sum)`
//
// 5. some of the parameters can be omitted, in this case, the parameters must
//    still be in their original order. e.g.
//
//   `(local.load32 $db (offset 0))` == `(local.load32 $db)`
//   ;; the child node '(offset ...)' above can be omitted.

// the instruction syntax
// ----------------------
//
// 1. instructions with NO parameters and NO operands, can be written
//    with or without parentheses, e.g.
//    '(nop)'
//    'nop'
//
// 2. instructions that have NO parameters but HAVE operands, should be
//    written with parentheses and all the operands (instructions) should be
//    written inside the parentheses, e.g.
//    '(i32.add zero zero)'
//
// 3. instructions WITH parameters must be written with parentheses, e.g.
//    '(i32.imm 0x1133)'
//    '(local.load $abc)'
//    '(local.load $abc 8)  ;; 8 is an optional parameter'
//
// 4. instructions that HAVE BOTH parameters and operands must be written
//    with parentheses, and the operands must be written after the parameters, e.g.
//    '(local.store $xyz (i32.imm 11))'
//
//    ```
//    (local.store $xyz
//        (i32.add
//            (i32.imm 11) (i32.imm 13)
//        )
//    )
//    ```

use ancvm_types::{opcode::Opcode, CompileError, DataType, MemoryDataType};

use crate::{
    ast::{FuncNode, Instruction, LocalNode, ModuleElementNode, ModuleNode, ParamNode},
    instruction_kind::{InstructionKind, INSTRUCTION_KIND_TABLE},
    lexer::Token,
    peekable_iterator::PeekableIterator,
};

pub fn parse(iter: &mut PeekableIterator<Token>) -> Result<ModuleNode, CompileError> {
    // there is only one node 'module' in a assembly text
    parse_module_node(iter)
}

pub fn parse_module_node(iter: &mut PeekableIterator<Token>) -> Result<ModuleNode, CompileError> {
    // (module ...) ...  //
    // ^            ^____// to here
    // |_________________// current token, i.e. the value of 'iter.peek(0)'

    // the node 'module' syntax:
    //
    // (module "name" (runtime_version "1.0") ...)  ;; base
    // (module "name" (runtime_version "1.0")
    //                                          ;; optional parameters
    //      (constructor $func_name)            ;; similar to GCC '__attribute__((constructor))', run before main()
    //      (entry $func_name)                  ;; similar to 'fn main()'
    //      (destructor $func_name)             ;; similar to GCC '__attribute__((destructor))', run after main()
    //      ...
    // )

    consume_left_paren(iter, "module")?;
    consume_symbol(iter, "module")?;

    let name = expect_string(iter, "module name")?;
    let (runtime_version_major, runtime_version_minor) = parse_module_runtime_version_node(iter)?;

    // optional parameters
    if expect_child_node_optional(iter, "constructor") {
        todo!()
    }
    if expect_child_node_optional(iter, "entry") {
        todo!()
    }
    if expect_child_node_optional(iter, "destructor") {
        todo!()
    }

    let mut element_nodes: Vec<ModuleElementNode> = vec![];

    // parse module elements
    while iter.look_ahead_equals(0, &Token::LeftParen) {
        if let Some(Token::Symbol(child_node_name)) = iter.peek(1) {
            let element_node = match child_node_name.as_str() {
                "fn" => parse_func_node(iter)?,
                _ => {
                    return Err(CompileError::new(&format!(
                        "Unknown module element: {}",
                        child_node_name
                    )))
                }
            };
            element_nodes.push(element_node);
        } else {
            break;
        }
    }

    consume_right_paren(iter)?;

    let module_node = ModuleNode {
        name,
        runtime_version_major,
        runtime_version_minor,
        shared_packages: vec![],
        element_nodes,
    };

    Ok(module_node)
}

fn parse_module_runtime_version_node(
    iter: &mut PeekableIterator<Token>,
) -> Result<(u16, u16), CompileError> {
    // (runtime_version "1.0") ...  //
    // ^                       ^____// to here
    // |____________________________// current token

    consume_left_paren(iter, "module runtime version")?;
    consume_symbol(iter, "runtime_version")?;
    let ver_string = expect_string(iter, "module runtime version")?;
    consume_right_paren(iter)?;

    let vers: Vec<&str> = ver_string.split('.').collect();
    if vers.len() != 2 {
        return Err(CompileError::new(&format!(
            "Error runtime version number, expect: \"major.minor\", actual: \"{}\"",
            ver_string
        )));
    }

    let major = vers[0].parse::<u16>().map_err(|_| {
        CompileError::new(&format!(
            "Major version '{}' is not a valid number.",
            vers[0]
        ))
    })?;

    let minor = vers[1].parse::<u16>().map_err(|_| {
        CompileError::new(&format!(
            "Minor version '{}' is not a valid number.",
            vers[1]
        ))
    })?;

    Ok((major, minor))
}

fn parse_func_node(iter: &mut PeekableIterator<Token>) -> Result<ModuleElementNode, CompileError> {
    // (fn ...) ...  //
    // ^              ^____// to here
    // |___________________// current token

    // the node 'fn' syntax:
    //
    // (fn $add (param $lhs i32) (param $rhs i32) (result i32) ...)         ;; signature
    // (fn $add (param $lhs i32) (param $rhs i32) (results i32 i32) ...)    ;; signature with multiple return values
    // (fn $add
    //     (local $sum i32)             ;; local variable with identifier and data type
    //     (local $db (bytes 12 4))     ;; bytes-type local variable
    //     ...
    // )
    //
    // (fn $add
    //     (code ...)                   ;; the function body, the instructions sequence, sholud be written inside the node '(code)'
    // )

    consume_left_paren(iter, "fn")?;
    consume_symbol(iter, "fn")?;
    let name = expect_identifier_optional(iter);
    let (params, results) = parse_optional_signature(iter)?;
    let locals: Vec<LocalNode> = parse_optional_local_variables(iter)?;
    let mut instructions: Vec<Instruction> = parse_instruction_sequence(iter, "code")?;
    consume_right_paren(iter)?;

    // function's code implies an instruction 'end' at the end.
    instructions.push(Instruction::NoParams(Opcode::end));

    let func_node = FuncNode {
        name,
        params,
        results,
        locals,
        instructions,
    };

    Ok(ModuleElementNode::FuncNode(func_node))
}

fn parse_optional_signature(
    iter: &mut PeekableIterator<Token>,
) -> Result<(Vec<ParamNode>, Vec<DataType>), CompileError> {
    // (param|result|results ...){0,} ...  //
    // ^                              ^____// to here
    // |___________________________________// current token

    let mut params: Vec<ParamNode> = vec![];
    let mut results: Vec<DataType> = vec![];

    while iter.look_ahead_equals(0, &Token::LeftParen) {
        if let Some(Token::Symbol(child_node_name)) = iter.peek(1) {
            match child_node_name.as_str() {
                "param" => {
                    let param_node = parse_param_node(iter)?;
                    params.push(param_node);
                }
                "result" => {
                    let data_type = parse_result_node(iter)?;
                    results.push(data_type);
                }
                "results" => {
                    let mut data_types = parse_results_node(iter)?;
                    results.append(&mut data_types);
                }
                _ => break,
            }
        } else {
            break;
        }
    }

    Ok((params, results))
}

fn parse_param_node(iter: &mut PeekableIterator<Token>) -> Result<ParamNode, CompileError> {
    // (param $name i32) ...  //
    // ^                 ^____// to here
    // |______________________// current token

    consume_left_paren(iter, "param")?;
    consume_symbol(iter, "param")?;
    let tag = expect_identifier(iter, "param")?;
    let data_type = parse_data_type(iter)?;
    consume_right_paren(iter)?;

    Ok(ParamNode { tag, data_type })
}

fn parse_result_node(iter: &mut PeekableIterator<Token>) -> Result<DataType, CompileError> {
    // (result i32) ...  //
    // ^            ^____// to here
    // |_________________// current token

    consume_left_paren(iter, "result")?;
    consume_symbol(iter, "result")?;
    let data_type = parse_data_type(iter)?;
    consume_right_paren(iter)?;

    Ok(data_type)
}

fn parse_results_node(iter: &mut PeekableIterator<Token>) -> Result<Vec<DataType>, CompileError> {
    // (results i32 i32 i64) ...  //
    // ^                     ^____// to here
    // |__________________________// current token

    let mut data_types: Vec<DataType> = vec![];

    consume_left_paren(iter, "results")?;
    consume_symbol(iter, "results")?;
    while matches!(iter.peek(0), &Some(Token::Symbol(_))) {
        let data_type = parse_data_type(iter)?;
        data_types.push(data_type);
    }

    consume_right_paren(iter)?;

    Ok(data_types)
}

fn parse_data_type(iter: &mut PeekableIterator<Token>) -> Result<DataType, CompileError> {
    // i32 ...  //
    // i64 ...  //
    // f32 ...  //
    // f64 ...  //
    // ^   ^____// to here
    // |________// current token

    let data_type_name = expect_symbol(iter, "data type")?;
    let data_type = match data_type_name.as_str() {
        "i32" => DataType::I32,
        "i64" => DataType::I64,
        "f32" => DataType::F32,
        "f64" => DataType::F64,
        _ => {
            return Err(CompileError::new(&format!(
                "Unknown data type: {}",
                data_type_name
            )))
        }
    };
    Ok(data_type)
}

fn parse_optional_local_variables(
    iter: &mut PeekableIterator<Token>,
) -> Result<Vec<LocalNode>, CompileError> {
    // (local $name i32){0,} ...  //
    // ^                     ^____// to here
    // |__________________________// current token

    let mut local_nodes: Vec<LocalNode> = vec![];

    while iter.look_ahead_equals(0, &Token::LeftParen) {
        if let Some(Token::Symbol(child_node_name)) = iter.peek(1) {
            match child_node_name.as_str() {
                "local" => {
                    let local_node = parse_local_node(iter)?;
                    local_nodes.push(local_node);
                }
                _ => break,
            }
        } else {
            break;
        }
    }

    Ok(local_nodes)
}

fn parse_local_node(iter: &mut PeekableIterator<Token>) -> Result<LocalNode, CompileError> {
    // (local $name i32) ...  //
    // ^                 ^____// to here
    // |______________________// current token

    consume_left_paren(iter, "local")?;
    consume_symbol(iter, "local")?;
    let tag = expect_identifier(iter, "local")?;

    let (memory_data_type, data_length, align) = if iter.look_ahead_equals(0, &Token::LeftParen) {
        parse_memory_data_type_bytes(iter)?
    } else {
        parse_memory_data_type_primitive(iter)?
    };

    consume_right_paren(iter)?;

    Ok(LocalNode {
        tag,
        memory_data_type,
        data_length,
        align,
    })
}

// return '(MemoryDataType, data length, align)'
fn parse_memory_data_type_primitive(
    iter: &mut PeekableIterator<Token>,
) -> Result<(MemoryDataType, u32, u16), CompileError> {
    // i32 ...  //
    // i64 ...  //
    // f32 ...  //
    // f64 ...  //
    // ^   ^____// to here
    // |________// current token

    let memory_data_type_name = expect_symbol(iter, "memory data type")?;
    let memory_data_type_detail = match memory_data_type_name.as_str() {
        "i32" => (MemoryDataType::I32, 4, 4),
        "i64" => (MemoryDataType::I64, 8, 8),
        "f32" => (MemoryDataType::F32, 4, 4),
        "f64" => (MemoryDataType::F64, 8, 8),
        _ => {
            return Err(CompileError::new(&format!(
                "Unknown memory data type: {}",
                memory_data_type_name
            )))
        }
    };

    Ok(memory_data_type_detail)
}

// return '(MemoryDataType, data length, align)'
fn parse_memory_data_type_bytes(
    iter: &mut PeekableIterator<Token>,
) -> Result<(MemoryDataType, u32, u16), CompileError> {
    // (bytes 12 8)) ...  //
    // ^             ^____// to here
    // |__________________// current token

    // the node 'bytes' syntax:
    //
    // (bytes length align)

    consume_left_paren(iter, "bytes")?;
    consume_symbol(iter, "bytes")?;

    let length_string = expect_number(iter, "the length of memory data type bytes")?;
    let align_string = expect_number(iter, "the align of memory data type bytes")?;

    let length = length_string.parse::<u32>().map_err(|_| {
        CompileError::new(&format!(
            "The length of memory data type bytes '{}' is not a valid number.",
            length_string
        ))
    })?;

    let align = align_string.parse::<u16>().map_err(|_| {
        CompileError::new(&format!(
            "The align of memory data type bytes '{}' is not a valid number.",
            align_string
        ))
    })?;

    if align == 0 || align > 8 {
        return Err(CompileError::new(&format!(
            "The range of align of memory data type bytes should be 1 to 8, actual: '{}'.",
            align
        )));
    }

    consume_right_paren(iter)?;

    Ok((MemoryDataType::BYTES, length, align))
}

fn parse_instruction_sequence(
    iter: &mut PeekableIterator<Token>,
    node_name: &str,
) -> Result<Vec<Instruction>, CompileError> {
    // (code ...) ...  //
    // ^          ^____// to here
    // ________________// current token
    //
    // similar instruction sequence nodes:
    //
    // - (recur ...)
    // - (tailcall ...)
    // - (break ...)
    // - (return ...)

    consume_left_paren(iter, node_name)?;
    consume_symbol(iter, node_name)?;
    let mut instructions = vec![];

    while let Some(mut sub_instructions) = parse_next_instruction_optional(iter)? {
        instructions.append(&mut sub_instructions);
    }

    consume_right_paren(iter)?;

    Ok(instructions)
}

fn parse_next_instruction_optional(
    iter: &mut PeekableIterator<Token>,
) -> Result<Option<Vec<Instruction>>, CompileError> {
    let instructions = if let Some(token) = iter.peek(0) {
        match token {
            Token::LeftParen => {
                // parse instruction WITH parentheses
                parse_instruction_with_parentheses(iter)?
            }
            Token::Symbol(_) => {
                // parse instruction WITHOUT parentheses
                vec![parse_instruction_without_parentheses(iter)?]
            }
            _ => return Ok(None),
        }
    } else {
        return Ok(None);
    };

    Ok(Some(instructions))
}

fn parse_next_instruction_operand(
    iter: &mut PeekableIterator<Token>,
    for_which_inst: &str,
) -> Result<Vec<Instruction>, CompileError> {
    let instructions = if let Some(token) = iter.peek(0) {
        match token {
            Token::LeftParen => {
                // parse instruction WITH parentheses
                parse_instruction_with_parentheses(iter)?
            }
            Token::Symbol(_) => {
                // parse instruction WITHOUT parentheses
                vec![parse_instruction_without_parentheses(iter)?]
            }
            _ => {
                return Err(CompileError::new(&format!(
                    "Expect operand for instruction \"{}\", actual {:?}",
                    for_which_inst, token
                )))
            }
        }
    } else {
        return Err(CompileError::new(&format!(
            "Missing operand for instruction \"{}\"",
            for_which_inst
        )));
    };

    Ok(instructions)
}

// parse the instruction with parentheses,
//
// ✖️: i32.add
// ✅: (i32.add ...)
//
fn parse_instruction_with_parentheses(
    iter: &mut PeekableIterator<Token>,
) -> Result<Vec<Instruction>, CompileError> {
    // (inst_name) ...  //
    // ^           ^____// to here
    // |________________// current token
    //
    // also maybe:
    //
    // (inst_name PARAM0 PARAM1 ...)
    // (inst_name OPERAND0 OPERAND1 ...)
    // (inst_name PARAM0 ... OPERAND0 ...)
    // (pesudo_inst_name ...)

    if let Some(Token::Symbol(child_node_name)) = iter.peek(1) {
        let owned_name = child_node_name.to_owned();
        let inst_name = owned_name.as_str();
        let instructions = if let Some(kind) = get_instruction_kind(inst_name) {
            match *kind {
                InstructionKind::NoParams(opcode, operand_count) => {
                    parse_instruction_kind_no_params(iter, inst_name, opcode, operand_count)?
                }
                //
                InstructionKind::LocalLoad(opcode) => {
                    parse_instruction_kind_local_load(iter, inst_name, opcode, true)?
                }
                InstructionKind::LocalStore(opcode) => {
                    parse_instruction_kind_local_store(iter, inst_name, opcode, true)?
                }
                InstructionKind::LocalLongLoad(opcode) => {
                    parse_instruction_kind_local_long_load(iter, inst_name, opcode, true)?
                }
                InstructionKind::LocalLongStore(opcode) => {
                    parse_instruction_kind_local_long_store(iter, inst_name, opcode, true)?
                }
                InstructionKind::DataLoad(opcode) => {
                    parse_instruction_kind_local_load(iter, inst_name, opcode, false)?
                }
                InstructionKind::DataStore(opcode) => {
                    parse_instruction_kind_local_store(iter, inst_name, opcode, false)?
                }
                InstructionKind::DataLongLoad(opcode) => {
                    parse_instruction_kind_local_long_load(iter, inst_name, opcode, false)?
                }
                InstructionKind::DataLongStore(opcode) => {
                    parse_instruction_kind_local_long_store(iter, inst_name, opcode, false)?
                }
                //
                InstructionKind::HeapLoad(opcode) => {
                    parse_instruction_kind_heap_load(iter, inst_name, opcode)?
                }
                InstructionKind::HeapStore(opcode) => {
                    parse_instruction_kind_heap_store(iter, inst_name, opcode)?
                }
                //
                InstructionKind::UnaryOp(opcode) => {
                    parse_instruction_kind_unary_op(iter, inst_name, opcode)?
                }
                InstructionKind::UnaryOpParamI16(opcode) => {
                    parse_instruction_kind_unary_op_param_i16(iter, inst_name, opcode)?
                }
                InstructionKind::BinaryOp(opcode) => {
                    parse_instruction_kind_binary_op(iter, inst_name, opcode)?
                }
                //
                InstructionKind::ImmI32 => vec![parse_instruction_kind_imm_i32(iter)?],
                InstructionKind::ImmI64 => vec![parse_instruction_kind_imm_i64(iter)?],
                InstructionKind::ImmF32 => todo!(),
                InstructionKind::ImmF64 => todo!(),
                //
                InstructionKind::When => todo!(),
                InstructionKind::If => todo!(),
                InstructionKind::Branch => todo!(),
                InstructionKind::Case => todo!(),
                InstructionKind::Default => todo!(),
                InstructionKind::For => todo!(),
                InstructionKind::Sequence(_) => todo!(),
                //
                InstructionKind::Call => todo!(),
                InstructionKind::DynCall => todo!(),
                InstructionKind::EnvCall => todo!(),
                InstructionKind::SysCall => todo!(),
                InstructionKind::ExtCall => todo!(),
            }
        } else {
            return Err(CompileError::new(&format!(
                "Unknown instruction: {}",
                child_node_name
            )));
        };

        Ok(instructions)
    } else {
        Err(CompileError::new("Missing symbol for instruction node."))
    }
}

// parse the instruction without parentheses,
// that is, the instruction has no_parameters and no operands.
//
// ✅: zero
// ✖️: (i32.add ...)
//
fn parse_instruction_without_parentheses(
    iter: &mut PeekableIterator<Token>,
) -> Result<Instruction, CompileError> {
    // zero ... //
    // ^    ^___// to here
    // |________// current token

    let node_name = expect_symbol(iter, "instruction")?;
    let inst_name = node_name.as_str();

    if let Some(kind) = get_instruction_kind(inst_name) {
        match kind {
            InstructionKind::NoParams(opcode, operand_cound) => {
                if *operand_cound > 0 {
                    Err(CompileError::new(&format!(
                        "Instruction \"{}\" has operands and should be written with parentheses.",
                        inst_name
                    )))
                } else {
                    Ok(Instruction::NoParams(*opcode))
                }
            }
            _ => Err(CompileError::new(&format!(
                "Instruction \"{}\" should have parameters.",
                inst_name
            ))),
        }
    } else {
        Err(CompileError::new(&format!(
            "Unknown instruction: {}",
            inst_name
        )))
    }
}

fn parse_instruction_kind_no_params(
    iter: &mut PeekableIterator<Token>,
    inst_name: &str,
    opcode: Opcode,
    operand_count: u8,
) -> Result<Vec<Instruction>, CompileError> {
    // (name) ...  //
    // ^      ^____// to here
    // |___________// current token
    //
    // also:
    // (name OPERAND0 ... OPERAND_N) ...  //
    // ^                             ^____// to here
    // |__________________________________// current token

    let mut instructions = vec![];

    consume_left_paren(iter, "instruction")?;
    consume_symbol(iter, inst_name)?;

    // operands
    for _ in 0..operand_count {
        let mut child_instructions = parse_next_instruction_operand(iter, inst_name)?;
        instructions.append(&mut child_instructions);
    }

    consume_right_paren(iter)?;

    // add main instruction at last
    instructions.push(Instruction::NoParams(opcode));

    Ok(instructions)
}

fn parse_instruction_kind_imm_i32(
    iter: &mut PeekableIterator<Token>,
) -> Result<Instruction, CompileError> {
    // (i32.imm 123) ... //
    // ^             ^___// to here
    // |_________________// current token

    consume_left_paren(iter, "i32.imm")?;
    consume_symbol(iter, "i32.imm")?;
    let num_string = expect_number(iter, "i32.imm")?;
    consume_right_paren(iter)?;

    Ok(Instruction::ImmI32(parse_u32_string(num_string)?))
}

fn parse_instruction_kind_imm_i64(
    iter: &mut PeekableIterator<Token>,
) -> Result<Instruction, CompileError> {
    // (i64.imm 123) ... //
    // ^             ^___// to here
    // |_________________// current token

    consume_left_paren(iter, "i64.imm")?;
    consume_symbol(iter, "i64.imm")?;
    let num_string = expect_number(iter, "i64.imm")?;
    consume_right_paren(iter)?;

    Ok(Instruction::ImmI64(parse_u64_string(num_string)?))
}

fn parse_instruction_kind_unary_op(
    iter: &mut PeekableIterator<Token>,
    inst_name: &str,
    opcode: Opcode,
) -> Result<Vec<Instruction>, CompileError> {
    // (i32.not OPERAND) ... //
    // ^                 ^___// to here
    // |_____________________// current token

    consume_left_paren(iter, "instruction")?;
    consume_symbol(iter, inst_name)?;
    let mut instructions = parse_next_instruction_operand(iter, inst_name)?;
    consume_right_paren(iter)?;

    // add main instruction at last
    instructions.push(Instruction::UnaryOp(opcode));

    Ok(instructions)
}

fn parse_instruction_kind_unary_op_param_i16(
    iter: &mut PeekableIterator<Token>,
    inst_name: &str,
    opcode: Opcode,
) -> Result<Vec<Instruction>, CompileError> {
    // (i32.inc num OPERAND) ... //
    // ^                     ^___// to here
    // |_________________________// current token

    consume_left_paren(iter, "instruction")?;
    consume_symbol(iter, inst_name)?;
    let num_string = expect_number(iter, inst_name)?;
    let param_i16 = parse_u16_string(num_string)?;
    let mut instructions = parse_next_instruction_operand(iter, inst_name)?;
    consume_right_paren(iter)?;

    // add main instruction at last
    instructions.push(Instruction::UnaryOpParamI16(opcode, param_i16));

    Ok(instructions)
}

fn parse_instruction_kind_binary_op(
    iter: &mut PeekableIterator<Token>,
    inst_name: &str,
    opcode: Opcode,
) -> Result<Vec<Instruction>, CompileError> {
    // (i32.add OPERAND_LHS OPERAND_RHS) ... //
    // ^                                 ^___// to here
    // |_____________________________________// current token

    consume_left_paren(iter, "instruction")?;
    consume_symbol(iter, inst_name)?;
    let mut lhs_instructions = parse_next_instruction_operand(iter, inst_name)?;
    let mut rhs_instructions = parse_next_instruction_operand(iter, inst_name)?;
    consume_right_paren(iter)?;

    let mut instructions = vec![];
    instructions.append(&mut lhs_instructions);
    instructions.append(&mut rhs_instructions);

    // add main instruction at last
    instructions.push(Instruction::BinaryOp(opcode));

    Ok(instructions)
}

fn parse_instruction_kind_local_load(
    iter: &mut PeekableIterator<Token>,
    inst_name: &str,
    opcode: Opcode,
    local: bool,
) -> Result<Vec<Instruction>, CompileError> {
    // (local.load $id) ... //
    // ^                ^___// to here
    // |____________________// current token
    //
    // also:
    // (local.load $id 8)

    consume_left_paren(iter, "instruction")?;
    consume_symbol(iter, inst_name)?;
    let id = expect_identifier(iter, inst_name)?;
    let offset = if let Some(offset_str) = expect_number_optional(iter) {
        parse_u16_string(offset_str)?
    } else {
        0
    };
    consume_right_paren(iter)?;

    let mut instructions = vec![];
    if local {
        instructions.push(Instruction::LocalAccess(opcode, id, offset));
    } else {
        instructions.push(Instruction::DataAccess(opcode, id, offset));
    }

    Ok(instructions)
}

fn parse_instruction_kind_local_store(
    iter: &mut PeekableIterator<Token>,
    inst_name: &str,
    opcode: Opcode,
    local: bool,
) -> Result<Vec<Instruction>, CompileError> {
    // (local.store $id OPERAND) ... //
    // ^                         ^___// to here
    // |_____________________________// current token
    //
    // also:
    // (local.store $id 8 OPERAND)

    consume_left_paren(iter, "instruction")?;
    consume_symbol(iter, inst_name)?;
    let id = expect_identifier(iter, inst_name)?;
    let offset = if let Some(offset_str) = expect_number_optional(iter) {
        parse_u16_string(offset_str)?
    } else {
        0
    };

    let mut instructions = parse_next_instruction_operand(iter, inst_name)?;
    consume_right_paren(iter)?;

    if local {
        instructions.push(Instruction::LocalAccess(opcode, id, offset));
    } else {
        instructions.push(Instruction::DataAccess(opcode, id, offset));
    }

    Ok(instructions)
}

fn parse_instruction_kind_local_long_load(
    iter: &mut PeekableIterator<Token>,
    inst_name: &str,
    opcode: Opcode,
    local: bool,
) -> Result<Vec<Instruction>, CompileError> {
    // (local.long_load $id OPERAND_FOR_OFFSET) ... //
    // ^                                        ^___// to here
    // |____________________________________________// current token

    consume_left_paren(iter, "instruction")?;
    consume_symbol(iter, inst_name)?;
    let id = expect_identifier(iter, inst_name)?;
    let mut instructions = parse_next_instruction_operand(iter, inst_name)?;
    consume_right_paren(iter)?;

    if local {
        instructions.push(Instruction::LocalLongAccess(opcode, id));
    } else {
        instructions.push(Instruction::DataLongAccess(opcode, id));
    }

    Ok(instructions)
}

fn parse_instruction_kind_local_long_store(
    iter: &mut PeekableIterator<Token>,
    inst_name: &str,
    opcode: Opcode,
    local: bool,
) -> Result<Vec<Instruction>, CompileError> {
    // (local.long_store $id OPERAND_FOR_OFFSET OPERAND) ... //
    // ^                                                 ^___// to here
    // |_____________________________________________________// current token

    consume_left_paren(iter, "instruction")?;
    consume_symbol(iter, inst_name)?;
    let id = expect_identifier(iter, inst_name)?;
    let mut offset_instructions = parse_next_instruction_operand(iter, inst_name)?;
    let mut operand_instructions = parse_next_instruction_operand(iter, inst_name)?;
    consume_right_paren(iter)?;

    let mut instructions = vec![];
    instructions.append(&mut offset_instructions);
    instructions.append(&mut operand_instructions);

    if local {
        instructions.push(Instruction::LocalLongAccess(opcode, id));
    } else {
        instructions.push(Instruction::DataLongAccess(opcode, id));
    }

    Ok(instructions)
}

fn parse_instruction_kind_heap_load(
    iter: &mut PeekableIterator<Token>,
    inst_name: &str,
    opcode: Opcode,
) -> Result<Vec<Instruction>, CompileError> {
    // (heap.load OPERAND_FOR_ADDR) ... //
    // ^                            ^___// to here
    // |________________________________// current token
    //
    // also:
    // (heap.load offset OPERAND_FOR_ADDR)

    consume_left_paren(iter, "instruction")?;
    consume_symbol(iter, inst_name)?;

    let offset = if let Some(offset_str) = expect_number_optional(iter) {
        parse_u16_string(offset_str)?
    } else {
        0
    };

    let mut instructions = parse_next_instruction_operand(iter, inst_name)?;
    consume_right_paren(iter)?;

    instructions.push(Instruction::HeapAccess(opcode, offset));

    Ok(instructions)
}

fn parse_instruction_kind_heap_store(
    iter: &mut PeekableIterator<Token>,
    inst_name: &str,
    opcode: Opcode,
) -> Result<Vec<Instruction>, CompileError> {
    // (heap.store OPERAND_FOR_ADDR OPERAND) ... //
    // ^                                     ^___// to here
    // |_________________________________________// current token
    //
    // also:
    // (heap.store offset OPERAND_FOR_ADDR OPERAND)

    consume_left_paren(iter, "instruction")?;
    consume_symbol(iter, inst_name)?;

    let offset = if let Some(offset_str) = expect_number_optional(iter) {
        parse_u16_string(offset_str)?
    } else {
        0
    };

    let mut addr_instructions = parse_next_instruction_operand(iter, inst_name)?;
    let mut operand_instructions = parse_next_instruction_operand(iter, inst_name)?;

    consume_right_paren(iter)?;

    let mut instructions = vec![];
    instructions.append(&mut addr_instructions);
    instructions.append(&mut operand_instructions);

    instructions.push(Instruction::HeapAccess(opcode, offset));

    Ok(instructions)
}

// helper functions

fn consume_token(
    iter: &mut PeekableIterator<Token>,
    expect_token: Token,
) -> Result<(), CompileError> {
    let opt_token = iter.next();
    if let Some(token) = opt_token {
        if token == expect_token {
            Ok(())
        } else {
            Err(CompileError::new(&format!(
                "Expect token: {:?}, actual token: {:?}",
                expect_token, token
            )))
        }
    } else {
        Err(CompileError::new(&format!(
            "Missing token: {:?}",
            expect_token
        )))
    }
}

fn consume_left_paren(
    iter: &mut PeekableIterator<Token>,
    for_what: &str,
) -> Result<(), CompileError> {
    let opt_token = iter.next();
    if let Some(token) = opt_token {
        if token == Token::LeftParen {
            Ok(())
        } else {
            Err(CompileError::new(&format!(
                "Expect a node for {}",
                for_what
            )))
        }
    } else {
        Err(CompileError::new(&format!(
            "Missing a node for: {}",
            for_what
        )))
    }
}

fn consume_right_paren(iter: &mut PeekableIterator<Token>) -> Result<(), CompileError> {
    consume_token(iter, Token::RightParen)
}

fn consume_symbol(iter: &mut PeekableIterator<Token>, name: &str) -> Result<(), CompileError> {
    consume_token(iter, Token::new_symbol(name))
}

fn expect_number(
    iter: &mut PeekableIterator<Token>,
    for_what: &str,
) -> Result<String, CompileError> {
    match iter.next() {
        Some(Token::Number(s)) => Ok(s),
        _ => Err(CompileError::new(&format!(
            "Expect a number for {}",
            for_what
        ))),
    }
}

fn expect_number_optional(iter: &mut PeekableIterator<Token>) -> Option<String> {
    match iter.peek(0) {
        Some(token) => {
            if let Token::Number(s) = token {
                let cs = s.clone();
                iter.next().unwrap();
                Some(cs)
            } else {
                None
            }
        }
        None => None,
    }
}

fn expect_string(
    iter: &mut PeekableIterator<Token>,
    for_what: &str,
) -> Result<String, CompileError> {
    match iter.next() {
        Some(Token::String_(s)) => Ok(s),
        _ => Err(CompileError::new(&format!(
            "Expect a string for {}",
            for_what
        ))),
    }
}

fn expect_symbol(
    iter: &mut PeekableIterator<Token>,
    for_what: &str,
) -> Result<String, CompileError> {
    match iter.next() {
        Some(token) => match token {
            Token::Symbol(s) => Ok(s),
            _ => Err(CompileError::new(&format!(
                "Expect a symbol for {}",
                for_what
            ))),
        },
        None => Err(CompileError::new(&format!(
            "Missing a symbol for {}",
            for_what
        ))),
    }
}

fn expect_identifier(
    iter: &mut PeekableIterator<Token>,
    for_what: &str,
) -> Result<String, CompileError> {
    match iter.next() {
        Some(token) => match token {
            Token::Identifier(s) => Ok(s),
            _ => Err(CompileError::new(&format!(
                "Expect a identifier for {}",
                for_what
            ))),
        },
        None => Err(CompileError::new(&format!(
            "Missing a identifier for {}",
            for_what
        ))),
    }
}

fn expect_identifier_optional(iter: &mut PeekableIterator<Token>) -> Option<String> {
    match iter.peek(0) {
        Some(token) => {
            if let Token::Identifier(s) = token {
                let cs = s.clone();
                iter.next().unwrap();
                Some(cs)
            } else {
                None
            }
        }
        None => None,
    }
}

fn expect_child_node_optional(iter: &mut PeekableIterator<Token>, child_node_name: &str) -> bool {
    if let Some(Token::LeftParen) = iter.peek(0) {
        matches!(iter.peek(1), Some(Token::Symbol(n)) if n == child_node_name)
    } else {
        false
    }
}

fn get_instruction_kind(inst_name: &str) -> Option<&InstructionKind> {
    unsafe {
        if let Some(table_ref) = &INSTRUCTION_KIND_TABLE {
            table_ref.get(inst_name)
        } else {
            panic!("The instruction table is not initialized yet.")
        }
    }
}

fn parse_u16_string(mut num_string: String) -> Result<u16, CompileError> {
    let e = CompileError::new(&format!(
        "\"{}\" is not a valid integer number.",
        num_string
    ));

    // remove underscores
    num_string.retain(|c| c != '_');

    let num = if num_string.starts_with("0x") {
        u16::from_str_radix(num_string.strip_prefix("0x").unwrap(), 16).map_err(|_| e)?
    } else if num_string.starts_with("0b") {
        u16::from_str_radix(num_string.strip_prefix("0b").unwrap(), 2).map_err(|_| e)?
    } else {
        num_string.as_str().parse::<i16>().map_err(|_| e)? as u16
    };

    Ok(num)
}

fn parse_u32_string(mut num_string: String) -> Result<u32, CompileError> {
    let e = CompileError::new(&format!(
        "\"{}\" is not a valid integer number.",
        num_string
    ));

    // remove underscores
    num_string.retain(|c| c != '_');

    let num = if num_string.starts_with("0x") {
        u32::from_str_radix(num_string.strip_prefix("0x").unwrap(), 16).map_err(|_| e)?
    } else if num_string.starts_with("0b") {
        u32::from_str_radix(num_string.strip_prefix("0b").unwrap(), 2).map_err(|_| e)?
    } else {
        num_string.as_str().parse::<i32>().map_err(|_| e)? as u32
    };

    Ok(num)
}

fn parse_u64_string(mut num_string: String) -> Result<u64, CompileError> {
    let e = CompileError::new(&format!(
        "\"{}\" is not a valid integer number.",
        num_string
    ));

    // remove underscores
    num_string.retain(|c| c != '_');

    let num = if num_string.starts_with("0x") {
        u64::from_str_radix(num_string.strip_prefix("0x").unwrap(), 16).map_err(|_| e)?
    } else if num_string.starts_with("0b") {
        u64::from_str_radix(num_string.strip_prefix("0b").unwrap(), 2).map_err(|_| e)?
    } else {
        num_string.as_str().parse::<i64>().map_err(|_| e)? as u64
    };

    Ok(num)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use ancvm_types::{opcode::Opcode, CompileError, DataType, MemoryDataType};

    use crate::{
        ast::{FuncNode, Instruction, LocalNode, ModuleElementNode, ModuleNode, ParamNode},
        instruction_kind::init_instruction_table,
        lexer::lex,
        peekable_iterator::PeekableIterator,
    };

    use super::parse;

    fn parse_from_str(s: &str) -> Result<ModuleNode, CompileError> {
        init_instruction_table();

        let mut chars = s.chars();
        let mut char_iter = PeekableIterator::new(&mut chars, 2);
        let mut tokens = lex(&mut char_iter)?.into_iter();
        let mut token_iter = PeekableIterator::new(&mut tokens, 2);
        parse(&mut token_iter)
    }

    #[test]
    fn test_parse_empty_module() {
        assert_eq!(
            parse_from_str(r#"(module "main" (runtime_version "1.2"))"#).unwrap(),
            ModuleNode {
                name: "main".to_owned(),
                runtime_version_major: 1,
                runtime_version_minor: 2,
                shared_packages: vec![],
                element_nodes: vec![]
            }
        );

        assert!(parse_from_str(r#"()"#).is_err());
        assert!(parse_from_str(r#"(module)"#).is_err());
        assert!(parse_from_str(r#"(module "main")"#).is_err());
        assert!(parse_from_str(r#"(module "main" ())"#).is_err());
        assert!(parse_from_str(r#"(module "main" (runtime_version))"#).is_err());
        assert!(parse_from_str(r#"(module "main" (runtime_version "1"))"#).is_err());
        assert!(parse_from_str(r#"(module "main" (runtime_version "1a.2b"))"#).is_err());
    }

    #[test]
    fn test_parse_function_node_signature() {
        assert_eq!(
            parse_from_str(
                r#"
            (module "main"
                (runtime_version "1.0")
                (fn $add (param $lhs i32) (param $rhs i64) (result i32) (result i64)
                    ;; no local variables
                    (code)
                )
            )
            "#
            )
            .unwrap(),
            ModuleNode {
                name: "main".to_owned(),
                runtime_version_major: 1,
                runtime_version_minor: 0,
                shared_packages: vec![],
                element_nodes: vec![ModuleElementNode::FuncNode(FuncNode {
                    name: Some("add".to_owned()),
                    params: vec![
                        ParamNode {
                            tag: "lhs".to_owned(),
                            data_type: DataType::I32
                        },
                        ParamNode {
                            tag: "rhs".to_owned(),
                            data_type: DataType::I64
                        }
                    ],
                    results: vec![DataType::I32, DataType::I64,],
                    locals: vec![],
                    instructions: vec![Instruction::NoParams(Opcode::end)]
                })]
            }
        );

        assert_eq!(
            parse_from_str(
                r#"
            (module "main"
                (runtime_version "1.0")
                (fn $add (param $lhs i32) (param $rhs i64) (results i32 i64)
                    ;; no local variables
                    (code)
                )
            )
            "#
            )
            .unwrap(),
            ModuleNode {
                name: "main".to_owned(),
                runtime_version_major: 1,
                runtime_version_minor: 0,
                shared_packages: vec![],
                element_nodes: vec![ModuleElementNode::FuncNode(FuncNode {
                    name: Some("add".to_owned()),
                    params: vec![
                        ParamNode {
                            tag: "lhs".to_owned(),
                            data_type: DataType::I32
                        },
                        ParamNode {
                            tag: "rhs".to_owned(),
                            data_type: DataType::I64
                        }
                    ],
                    results: vec![DataType::I32, DataType::I64,],
                    locals: vec![],
                    instructions: vec![Instruction::NoParams(Opcode::end)]
                })]
            }
        );
    }

    #[test]
    fn test_parse_function_node_local_variables() {
        assert_eq!(
            parse_from_str(
                r#"
            (module "main"
                (runtime_version "1.0")
                (fn $add
                    ;; no params and results
                    (local $sum i32) (local $count i64) (local $db (bytes 12 8)) (local $average f32)
                    (code)
                )
            )
            "#
            )
            .unwrap(),
            ModuleNode {
                name: "main".to_owned(),
                runtime_version_major: 1,
                runtime_version_minor: 0,
                shared_packages: vec![],
                element_nodes: vec![ModuleElementNode::FuncNode(FuncNode {
                    name: Some("add".to_owned()),
                    params: vec![],
                    results: vec![],
                    locals: vec![
                        LocalNode {
                            tag: "sum".to_owned(),
                            memory_data_type: MemoryDataType::I32,
                            data_length: 4,
                            align: 4
                        },
                        LocalNode {
                            tag: "count".to_owned(),
                            memory_data_type: MemoryDataType::I64,
                            data_length: 8,
                            align: 8
                        },
                        LocalNode {
                            tag: "db".to_owned(),
                            memory_data_type: MemoryDataType::BYTES,
                            data_length: 12,
                            align: 8
                        },
                        LocalNode {
                            tag: "average".to_owned(),
                            memory_data_type: MemoryDataType::F32,
                            data_length: 4,
                            align: 4
                        },
                    ],
                    instructions: vec![Instruction::NoParams(Opcode::end)]
                })]
            }
        );
    }

    fn parse_instructions_from_str(text: &str) -> Vec<Instruction> {
        let module_node = parse_from_str(text).unwrap();
        if let ModuleElementNode::FuncNode(func_node) = &module_node.element_nodes[0] {
            func_node.instructions.clone()
        } else {
            panic!("Expect function node")
        }
    }

    #[test]
    fn test_parse_function_with_instructions_base() {
        assert_eq!(
            parse_instructions_from_str(
                r#"
            (module "lib"
                (runtime_version "1.0")
                (fn $test
                    (code
                        nop
                        (drop zero)
                    )
                )
            )
            "#
            ),
            vec![
                Instruction::NoParams(Opcode::nop),
                Instruction::NoParams(Opcode::zero),
                Instruction::NoParams(Opcode::drop),
                Instruction::NoParams(Opcode::end)
            ]
        );

        assert_eq!(
            parse_instructions_from_str(
                r#"
            (module "lib"
                (runtime_version "1.0")
                (fn $test
                    (code
                        (i32.imm 11)
                        (i32.imm 0x13)
                        (i32.imm 17_19)
                        (i32.imm -23)
                        (i32.imm 0xaa_bb)
                        (i32.imm 0b0110_0101)    ;; 101

                        (i64.imm 31)
                        (i64.imm 0x37)
                        (i64.imm 41_43)
                        (i64.imm -47)
                        (i64.imm 0xaabb_ccdd)
                        (i64.imm 0b0110_0111)   ;; 103
                    )
                )
            )
            "#
            ),
            vec![
                Instruction::ImmI32(11),
                Instruction::ImmI32(0x13),
                Instruction::ImmI32(17_19),
                Instruction::ImmI32((-23i32) as u32),
                Instruction::ImmI32(0xaa_bb),
                Instruction::ImmI32(0b0110_0101),
                Instruction::ImmI64(31),
                Instruction::ImmI64(0x37),
                Instruction::ImmI64(41_43),
                Instruction::ImmI64((-47i64) as u64),
                Instruction::ImmI64(0xaabb_ccdd),
                Instruction::ImmI64(0b0110_0111),
                Instruction::NoParams(Opcode::end)
            ]
        );

        assert_eq!(
            parse_instructions_from_str(
                r#"
            (module "lib"
                (runtime_version "1.0")
                (fn $test
                    (code
                        (i32.eqz (i32.imm 11))
                        (i32.inc 1 (i32.imm 13))
                        (i32.add (i32.imm 17) (i32.imm 19))
                        (i32.add
                            (i32.mul
                                (i32.imm 2)
                                (i32.imm 3)
                            )
                            (i32.imm 1)
                        )
                    )
                )
            )
            "#
            ),
            vec![
                // 11 == 0
                Instruction::ImmI32(11),
                Instruction::UnaryOp(Opcode::i32_eqz),
                // 13 + 1
                Instruction::ImmI32(13),
                Instruction::UnaryOpParamI16(Opcode::i32_inc, 1),
                // 17 + 19
                Instruction::ImmI32(17),
                Instruction::ImmI32(19),
                Instruction::BinaryOp(Opcode::i32_add),
                // 1 + 2 * 3
                Instruction::ImmI32(2),
                Instruction::ImmI32(3),
                Instruction::BinaryOp(Opcode::i32_mul),
                Instruction::ImmI32(1),
                Instruction::BinaryOp(Opcode::i32_add),
                // end
                Instruction::NoParams(Opcode::end)
            ]
        );
    }

    #[test]
    fn test_parse_function_with_instructions_local_and_data() {
        assert_eq!(
            parse_instructions_from_str(
                r#"
            (module "lib"
                (runtime_version "1.0")
                (fn $test
                    (code
                        (local.load32 $sum)
                        (local.load $count 4)
                        (local.store32 $left (i32.imm 11))
                        (local.store $right 8 (i64.imm 13))
                        (local.long_load $foo (i32.imm 17))
                        (local.long_store $bar (i32.imm 19) (i64.imm 23))
                    )
                )
            )
            "#
            ),
            vec![
                Instruction::LocalAccess(Opcode::local_load32, "sum".to_owned(), 0),
                Instruction::LocalAccess(Opcode::local_load, "count".to_owned(), 4),
                //
                Instruction::ImmI32(11),
                Instruction::LocalAccess(Opcode::local_store32, "left".to_owned(), 0),
                //
                Instruction::ImmI64(13),
                Instruction::LocalAccess(Opcode::local_store, "right".to_owned(), 8),
                //
                Instruction::ImmI32(17),
                Instruction::LocalLongAccess(Opcode::local_long_load, "foo".to_owned()),
                //
                Instruction::ImmI32(19),
                Instruction::ImmI64(23),
                Instruction::LocalLongAccess(Opcode::local_long_store, "bar".to_owned()),
                // end
                Instruction::NoParams(Opcode::end)
            ]
        );

        assert_eq!(
            parse_instructions_from_str(
                r#"
            (module "lib"
                (runtime_version "1.0")
                (fn $test
                    (code
                        (data.load32 $sum)
                        (data.load $count 4)
                        (data.store32 $left (i32.imm 11))
                        (data.store $right 8 (i64.imm 13))
                        (data.long_load $foo (i32.imm 17))
                        (data.long_store $bar (i32.imm 19) (i64.imm 23))
                    )
                )
            )
            "#
            ),
            vec![
                Instruction::DataAccess(Opcode::data_load32, "sum".to_owned(), 0),
                Instruction::DataAccess(Opcode::data_load, "count".to_owned(), 4),
                //
                Instruction::ImmI32(11),
                Instruction::DataAccess(Opcode::data_store32, "left".to_owned(), 0),
                //
                Instruction::ImmI64(13),
                Instruction::DataAccess(Opcode::data_store, "right".to_owned(), 8),
                //
                Instruction::ImmI32(17),
                Instruction::DataLongAccess(Opcode::data_long_load, "foo".to_owned()),
                //
                Instruction::ImmI32(19),
                Instruction::ImmI64(23),
                Instruction::DataLongAccess(Opcode::data_long_store, "bar".to_owned()),
                // end
                Instruction::NoParams(Opcode::end)
            ]
        );
    }

    #[test]
    fn test_parse_function_with_instructions_heap() {
        assert_eq!(
            parse_instructions_from_str(
                r#"
            (module "lib"
                (runtime_version "1.0")
                (fn $test
                    (code
                        (heap.load32 (i32.imm 11))
                        (heap.load 4 (i32.imm 13))
                        (heap.store32 (i32.imm 17) (i32.imm 19))
                        (heap.store 8 (i32.imm 23) (i32.imm 29))
                    )
                )
            )
            "#
            ),
            vec![
                Instruction::ImmI32(11),
                Instruction::HeapAccess(Opcode::heap_load32, 0),
                Instruction::ImmI32(13),
                Instruction::HeapAccess(Opcode::heap_load, 4),
                //
                Instruction::ImmI32(17),
                Instruction::ImmI32(19),
                Instruction::HeapAccess(Opcode::heap_store32, 0),
                //
                Instruction::ImmI32(23),
                Instruction::ImmI32(29),
                Instruction::HeapAccess(Opcode::heap_store, 8),
                // end
                Instruction::NoParams(Opcode::end)
            ]
        );
    }
}
