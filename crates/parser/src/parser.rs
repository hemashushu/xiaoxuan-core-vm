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
// 1. instructions with no parameters and no operands, can be written
//    with or without parentheses, e.g.
//    '(nop)'
//    'nop'
//
// 2. instructions that have no parameters but have operands, should be
//    written with parentheses and the operands (instructions) should be
//    written inside the parentheses, e.g.
//    '(i32.add zero zero)'
//
//    however, it also can be written without parentheses, but all operands
//    (instructions) must be expanded first, e.g.
//    'zero zero i32.add'
//
// 3. instructions with parameters must be written with parentheses, e.g.
//    '(i32.imm 0x1133)'
//    '(local.load $abc)'
//    '(local.load $abc 8)  ;; 8 is an optional parameter'
//
// 4. instructions that have both parameters and operands must be written
//    inside the parentheses and must be written after the parameters, e.g.
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
    instruction_property::{InstructionKind, InstructionProperty, INSTRUCTION_PROPERTY_TABLE},
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
    if exist_child_node(iter, "constructor") {
        todo!()
    }
    if exist_child_node(iter, "entry") {
        todo!()
    }
    if exist_child_node(iter, "destructor") {
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

    let major = u16::from_str_radix(vers[0], 10).map_err(|_| {
        CompileError::new(&format!(
            "Major version '{}' is not a valid number.",
            vers[0]
        ))
    })?;

    let minor = u16::from_str_radix(vers[1], 10).map_err(|_| {
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
    // (fn $add (param $lhs i32) (param $rhs i32) (result i32) ...)   ;; signature form 1
    // (fn $one
    //     (local $sum i32)             ;; local variable with identifier and data type
    //     (local $db (bytes 12 4))     ;; bytes-type local variable
    //     ...
    // )
    //
    // (fn $two
    //     (code ...)                   ;; the function body -- the instructions sequence
    // )

    consume_left_paren(iter, "fn")?;
    consume_symbol(iter, "fn")?;
    let name = maybe_identifier(iter);
    let (params, results) = parse_optional_signature(iter)?;
    let locals: Vec<LocalNode> = parse_optional_local_variables(iter)?;
    let mut instructions: Vec<Instruction> = parse_instructions_node(iter, "code")?;
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
    // ^                                     ^____// to here
    // |__________________________________________// current token

    let mut params: Vec<ParamNode> = vec![];
    let mut results: Vec<DataType> = vec![];

    while iter.look_ahead_equals(0, &Token::LeftParen) {
        if let Some(Token::Symbol(child_node_name)) = iter.peek(1) {
            match child_node_name.as_str() {
                "param" => {
                    let param_node = parse_param_node(iter)?;
                    params.push(param_node);
                }
                // "params" => {
                //     let data_types = parse_params_node(iter)?;
                //     let mut param_nodes = data_types
                //         .iter()
                //         .map(|dt| ParamNode {
                //             name: None,
                //             data_type: *dt,
                //         })
                //         .collect::<Vec<_>>();
                //     params.append(&mut param_nodes);
                // }
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

// fn parse_params_node(iter: &mut PeekableIterator<Token>) -> Result<Vec<DataType>, CompileError> {
//     // (params i32 i32 i64)  //
//     // ^           ^_________// to here
//     // |_____________________// current token
//
//     let mut data_types: Vec<DataType> = vec![];
//
//     consume_left_paren(iter, "params")?;
//     consume_symbol(iter, "params")?;
//     while matches!(iter.peek(0), &Some(Token::Symbol(_))) {
//         let data_type = parse_data_type(iter)?;
//         data_types.push(data_type);
//     }
//
//     consume_right_paren(iter)?;
//
//     Ok(data_types)
// }

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
    // (results i32 i32 i64)  //
    // ^            ^_________// to here
    // |______________________// current token

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
    // (local|locals ...){0,} ...  //
    // ^                      ^____// to here
    // |___________________________// current token

    let mut local_nodes: Vec<LocalNode> = vec![];

    while iter.look_ahead_equals(0, &Token::LeftParen) {
        if let Some(Token::Symbol(child_node_name)) = iter.peek(1) {
            match child_node_name.as_str() {
                "local" => {
                    let local_node = parse_local_node(iter)?;
                    local_nodes.push(local_node);
                }
                // "locals" => {
                //     let mut partial_local_nodes = parse_locals_node(iter)?;
                //     local_nodes.append(&mut partial_local_nodes);
                // }
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

// fn parse_locals_node(iter: &mut PeekableIterator<Token>) -> Result<Vec<LocalNode>, CompileError> {
//     // (locals i32 i64 f32 f64) ...  //
//     // ^                        ^____// to here
//     // |_____________________________// current token
//
//     let mut local_nodes: Vec<LocalNode> = vec![];
//
//     consume_left_paren(iter, "locals")?;
//     consume_symbol(iter, "locals")?;
//
//     loop {
//         if let Some(token) = iter.peek(0) {
//             let (memory_data_type, data_length, align) = match token {
//                 Token::LeftParen => parse_memory_data_type_bytes(iter)?,
//                 Token::Symbol(_) => parse_memory_data_type_primitive(iter)?,
//                 _ => break,
//             };
//
//             local_nodes.push(LocalNode {
//                 name: None,
//                 memory_data_type,
//                 data_length,
//                 align,
//             });
//         } else {
//             break;
//         }
//     }
//
//     consume_right_paren(iter)?;
//
//     Ok(local_nodes)
// }

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

    let length = u32::from_str_radix(&length_string, 10).map_err(|_| {
        CompileError::new(&format!(
            "The length of memory data type bytes '{}' is not a valid number.",
            length_string
        ))
    })?;

    let align = u16::from_str_radix(&align_string, 10).map_err(|_| {
        CompileError::new(&format!(
            "The align of memory data type bytes '{}' is not a valid number.",
            align_string
        ))
    })?;

    if align <= 0 || align > 8 {
        return Err(CompileError::new(&format!(
            "The range of align of memory data type bytes should be 1 to 8, actual: '{}'.",
            align
        )));
    }

    consume_right_paren(iter)?;

    Ok((MemoryDataType::BYTES, length, align))
}

fn parse_instructions_node(
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

    loop {
        if let Some(mut sub_instructions) = parse_instruction_optional(iter)? {
            instructions.append(&mut sub_instructions);
        } else {
            break;
        }
    }
    consume_right_paren(iter)?;

    Ok(instructions)
}

/// parse the instruction which have parentheses,
///
/// ✖️: i32.add
/// ✅: (i32.add)
/// ✅: (i32.add (i32.imm 11) (i32.imm 13))
///
fn parse_instruction_node(
    iter: &mut PeekableIterator<Token>,
) -> Result<Vec<Instruction>, CompileError> {
    // (inst_name) ...  //
    // ^           ^____// to here
    // |________________// current token
    //
    // also maybe:
    //
    // (inst_name param0 param1 ...)
    // (inst_name (param_node ...) ...)
    // (inst_name (sub_inst_name ...) ...)
    // (pesudo_inst ...)

    if let Some(Token::Symbol(child_node_name)) = iter.peek(1) {
        let instructions = if let Some(prop) = get_instruction_property(&child_node_name) {
            match prop.kind {
                InstructionKind::NoParams => parse_instruction_node_that_no_params(iter)?,
                InstructionKind::ParamI32 => todo!(),
                InstructionKind::ParamI16 => todo!(),
                InstructionKind::ImmI64 => todo!(),
                InstructionKind::ImmF32 => todo!(),
                InstructionKind::ImmF64 => todo!(),
                InstructionKind::LocalAccess => todo!(),
                InstructionKind::LocalAccessLong => todo!(),
                InstructionKind::DataAccess => todo!(),
                InstructionKind::DataAccessLong => todo!(),
                InstructionKind::HeapAccess => todo!(),
                InstructionKind::UnaryOp => todo!(),
                InstructionKind::BinaryOp => todo!(),
                InstructionKind::If => todo!(),
                InstructionKind::Cond => todo!(),
                InstructionKind::Branch => todo!(),
                InstructionKind::Case => todo!(),
                InstructionKind::Default => todo!(),
                InstructionKind::For => todo!(),
                InstructionKind::Sequence => todo!(),
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

/// parse the instruction which has no parentheses,
/// that is, the instruction has no parameters, and it may
/// have operands but it is not written in the folded form.
///
/// ✅: i32.add
/// ✖️: (i32.add)
/// ✖️: (i32.add (i32.imm 11) (i32.imm 13))
///
fn parse_instruction_without_parentheses(
    iter: &mut PeekableIterator<Token>,
) -> Result<Instruction, CompileError> {
    // i32.add ... //
    // ^       ^___// to here
    // |___________// current token

    let inst_name = expect_symbol(iter, "instruction")?;

    let instruction = if let Some(prop) = get_instruction_property(&inst_name) {
        match prop.kind {
            InstructionKind::NoParams => Instruction::NoParams(prop.opcode),
            _ => {
                return Err(CompileError::new(&format!(
                    "Instruction \"{}\" should have parameters.",
                    inst_name
                )))
            }
        }
    } else {
        return Err(CompileError::new(&format!(
            "Unknown instruction: {}",
            inst_name
        )));
    };

    Ok(instruction)
}

fn parse_instruction_node_that_no_params(
    iter: &mut PeekableIterator<Token>,
) -> Result<Vec<Instruction>, CompileError> {
    // (name) ...  //
    // ^      ^____// to here
    // |___________// current token
    //
    // (name (sub_inst ...) (...)) ...  //
    // ^                           ^____// to here
    // |________________________________// current token

    let mut instructions = vec![];

    consume_left_paren(iter, "instruction")?;

    let inst_name = expect_symbol(iter, "instruction")?;

    let (main_instruction, operand_count) = if let Some(prop) = get_instruction_property(&inst_name)
    {
        match prop.kind {
            InstructionKind::NoParams => (Instruction::NoParams(prop.opcode), prop.operand_count),
            _ => {
                return Err(CompileError::new(&format!(
                    "Instruction \"{}\" should have parameters.",
                    inst_name
                )))
            }
        }
    } else {
        return Err(CompileError::new(&format!(
            "Unknown instruction: {}",
            inst_name
        )));
    };

    // operands
    for _ in 0..operand_count {
        let mut sub_instructions = parse_instruction_operand(iter, &inst_name)?;
        instructions.append(&mut sub_instructions);
    }

    consume_right_paren(iter)?;

    // add main instruction at last
    instructions.push(main_instruction);

    Ok(instructions)
}

fn parse_instruction_optional(
    iter: &mut PeekableIterator<Token>,
) -> Result<Option<Vec<Instruction>>, CompileError> {
    let instructions = if let Some(token) = iter.peek(0) {
        match token {
            Token::LeftParen => {
                // parse instruction WITH parentheses
                parse_instruction_node(iter)?
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

fn parse_instruction_operand(
    iter: &mut PeekableIterator<Token>,
    for_which_inst: &str,
) -> Result<Vec<Instruction>, CompileError> {
    let instructions = if let Some(token) = iter.peek(0) {
        match token {
            Token::LeftParen => {
                // parse instruction WITH parentheses
                parse_instruction_node(iter)?
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
        Some(token) => match token {
            Token::Number(s) => Ok(s),
            _ => Err(CompileError::new(&format!(
                "Expect a number for {}",
                for_what
            ))),
        },
        None => Err(CompileError::new(&format!(
            "Expect a number for {}",
            for_what
        ))),
    }
}

fn expect_string(
    iter: &mut PeekableIterator<Token>,
    for_what: &str,
) -> Result<String, CompileError> {
    match iter.next() {
        Some(token) => match token {
            Token::String_(s) => Ok(s),
            _ => Err(CompileError::new(&format!(
                "Expect a string for {}",
                for_what
            ))),
        },
        None => Err(CompileError::new(&format!(
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

fn maybe_identifier(iter: &mut PeekableIterator<Token>) -> Option<String> {
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

fn exist_child_node(iter: &mut PeekableIterator<Token>, name: &str) -> bool {
    if let Some(Token::LeftParen) = iter.peek(0) {
        matches!(iter.peek(1), Some(Token::Symbol(n)) if n == name)
    } else {
        false
    }
}

fn get_instruction_property(inst_name: &str) -> Option<&InstructionProperty> {
    unsafe {
        if let Some(table_ref) = &INSTRUCTION_PROPERTY_TABLE {
            table_ref.get(inst_name)
        } else {
            panic!("The instruction table is not initialized yet.")
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use ancvm_types::{opcode::Opcode, CompileError, DataType, MemoryDataType};

    use crate::{
        ast::{FuncNode, Instruction, LocalNode, ModuleElementNode, ModuleNode, ParamNode},
        instruction_property::init_instruction_table,
        lexer::lex,
        peekable_iterator::PeekableIterator,
    };

    use super::parse;

    fn parse_from_str(s: &str) -> Result<ModuleNode, CompileError> {
        init_instruction_table();

        let mut chars = s.chars();
        let mut char_iter = PeekableIterator::new(&mut chars, 1);
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

        assert!(matches!(parse_from_str(r#"()"#), Err(_)));
        assert!(matches!(parse_from_str(r#"(module)"#), Err(_)));
        assert!(matches!(parse_from_str(r#"(module "main")"#), Err(_)));
        assert!(matches!(parse_from_str(r#"(module "main" ())"#), Err(_)));
        assert!(matches!(
            parse_from_str(r#"(module "main" (runtime_version))"#),
            Err(_)
        ));
        assert!(matches!(
            parse_from_str(r#"(module "main" (runtime_version "1"))"#),
            Err(_)
        ));
        assert!(matches!(
            parse_from_str(r#"(module "main" (runtime_version "1a.2b"))"#),
            Err(_)
        ));
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

        // assert_eq!(
        //     parse_from_str(
        //         r#"
        //     (module "main"
        //         (runtime_version "1.0")
        //         (fn $add
        //             ;; no params and results
        //             (locals i32 i64 (bytes 12 8) f32)
        //             (code)
        //         )
        //     )
        //     "#
        //     )
        //     .unwrap(),
        //     ModuleNode {
        //         name: "main".to_owned(),
        //         runtime_version_major: 1,
        //         runtime_version_minor: 0,
        //         shared_packages: vec![],
        //         element_nodes: vec![ModuleElementNode::FuncNode(FuncNode {
        //             name: Some("add".to_owned()),
        //             params: vec![],
        //             results: vec![],
        //             locals: vec![
        //                 LocalNode {
        //                     name: None,
        //                     memory_data_type: MemoryDataType::I32,
        //                     data_length: 4,
        //                     align: 4
        //                 },
        //                 LocalNode {
        //                     name: None,
        //                     memory_data_type: MemoryDataType::I64,
        //                     data_length: 8,
        //                     align: 8
        //                 },
        //                 LocalNode {
        //                     name: None,
        //                     memory_data_type: MemoryDataType::BYTES,
        //                     data_length: 12,
        //                     align: 8
        //                 },
        //                 LocalNode {
        //                     name: None,
        //                     memory_data_type: MemoryDataType::F32,
        //                     data_length: 4,
        //                     align: 4
        //                 },
        //             ],
        //             instructions: vec![Instruction::NoParams(Opcode::end)]
        //         })]
        //     }
        // );
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
    fn test_parse_function_with_base_instructions() {
        assert_eq!(
            parse_instructions_from_str(
                r#"
            (module "lib"
                (runtime_version "1.0")
                (fn $test
                    (code
                        nop
                        zero
                    )
                )
            )
            "#
            ),
            vec![
                Instruction::NoParams(Opcode::nop),
                Instruction::NoParams(Opcode::zero),
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
                        nop
                        (i32.add (i32.mul zero zero) zero)
                    )
                )
            )
            "#
            ),
            vec![
                Instruction::NoParams(Opcode::nop),
                // zero * zero
                Instruction::NoParams(Opcode::zero),
                Instruction::NoParams(Opcode::zero),
                Instruction::NoParams(Opcode::i32_mul),
                // zero + ?
                Instruction::NoParams(Opcode::zero),
                Instruction::NoParams(Opcode::i32_add),
                // end
                Instruction::NoParams(Opcode::end)
            ]
        );
    }
}
