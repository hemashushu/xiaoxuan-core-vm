// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// the fundanmental of XiaoXuan Core VM Assembly s-expression:
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
//    (func $id (param $lhs i32) (param $rhs i32) (result i32)
//        (body
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
//   `(local.load32 (index 0 0) (offset 0)` == `(local.load32 (index 0 0)`
//   ;; the child node '(offset ...)' above can be omitted.
//
//   `(local.load32 $db (offset 4)`
//   ;; the above is the another form of node '(local.load32 ...)', the child node
//   ;; '(offset ...)' must still be the last parameter.

use ancvm_types::{CompileError, DataType, MemoryDataType};

use crate::{
    ast::{FuncNode, LocalNode, ModuleElementNode, ModuleNode, ParamNode},
    lexer::Token,
    peekable_iterator::PeekableIterator,
};

pub fn parse(iter: &mut PeekableIterator<Token>) -> Result<ModuleNode, CompileError> {
    // the node 'module' syntax:
    //
    // ```clojure
    //
    // (module "name" (runtime_version "1.0") ...)  ;; base
    //
    // (module "name" (runtime_version "1.0")
    //                                          ;; optional parameters
    //      (constructor $func_name)            ;; similar to GCC '__attribute__((constructor))', run before main()
    //      (entry $func_name)                  ;; similar to 'function main()'
    //      (destructor $func_name)             ;; similar to GCC '__attribute__((destructor))', run after main()
    //      ...
    // )
    // ````

    consume_left_paren(iter, "module")?;
    consume_symbol(iter, "module")?;

    let name = expect_string(iter, "module name")?;
    let (runtime_version_major, runtime_version_minor) = parse_module_runtime_version(iter)?;
    let mut element_nodes: Vec<ModuleElementNode> = vec![];

    // parse module elements
    while iter.look_ahead(0, &Token::LeftParen) {
        consume_left_paren(iter, "module element")?;
        let element_node_name = expect_symbol(iter, "module element")?;
        let element_node = match element_node_name.as_str() {
            "func" => parse_func(iter)?,
            _ => {
                return Err(CompileError::new(&format!(
                    "Unknown module element: {}",
                    element_node_name
                )))
            }
        };
        element_nodes.push(element_node);
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

fn parse_module_runtime_version(
    iter: &mut PeekableIterator<Token>,
) -> Result<(u16, u16), CompileError> {
    // (runtime_version "1.0")  //
    // ^________________________// current token, i.e. the value of 'iter.peek(0)'

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

fn parse_func(iter: &mut PeekableIterator<Token>) -> Result<ModuleElementNode, CompileError> {
    // (func ...)  //
    //       ^_____// current token

    // the node 'func' syntax:
    //
    // (func $add (param $lhs i32) (param $rhs i32) (result i32) ...)   ;; signature form1
    // (func $add (params i32 i32 i32) (results i32 i32) ...)           ;; signature form2
    // (func $one
    //     (local $sum i32)             ;; local variable with identifier and data type
    //     (local i32)                  ;; local variable with data type only
    //     (local $db (bytes 12 4))     ;; bytes-type local variable
    //                                  ;; the node 'bytes' syntax: '(bytes length align)'
    //     (locals i32 i32)             ;; multiple local variables
    // )
    //
    // (func $two (code ...))           ;; the function body -- the instructions sequence

    let name = maybe_identifier(iter);
    let mut params: Vec<ParamNode> = vec![];
    let mut results: Vec<DataType> = vec![];
    let mut locals: Vec<LocalNode> = vec![];

    // parse params, results and local variables
    while iter.look_ahead(0, &Token::LeftParen) {
        consume_left_paren(iter, "function element")?;
        let element_node_name = expect_symbol(iter, "function element")?;
        match element_node_name.as_str() {
            "param" => {
                let param_node = parse_param_node(iter)?;
                params.push(param_node);
            }
            "params" => {
                let data_types = parse_params_node(iter)?;
                let mut param_nodes = data_types
                    .iter()
                    .map(|dt| ParamNode {
                        name: None,
                        data_type: *dt,
                    })
                    .collect::<Vec<_>>();
                params.append(&mut param_nodes);
            }
            "result" => {
                let data_type = parse_result_node(iter)?;
                results.push(data_type);
            }
            "results" => {
                let mut data_types = parse_results_node(iter)?;
                results.append(&mut data_types);
            }
            "local" => {
                let local_node = parse_local_node(iter)?;
                locals.push(local_node);
            }
            "locals" => {
                let mut local_nodes = parse_locals_node(iter)?;
                locals.append(&mut local_nodes);
            }
            _ => {
                return Err(CompileError::new(&format!(
                    "Unknown function element: {}",
                    element_node_name
                )))
            }
        };
    }

    consume_right_paren(iter)?;

    let func_node = FuncNode {
        name,
        params,
        results,
        locals,
    };

    Ok(ModuleElementNode::FuncNode(func_node))
}

fn parse_param_node(iter: &mut PeekableIterator<Token>) -> Result<ParamNode, CompileError> {
    // (param $name i32)  //
    //        ^___________// current token
    let name = maybe_identifier(iter);
    let data_type = parse_data_type(iter)?;
    consume_right_paren(iter)?;

    Ok(ParamNode { name, data_type })
}

fn parse_params_node(iter: &mut PeekableIterator<Token>) -> Result<Vec<DataType>, CompileError> {
    // (params i32 i32 i64)  //
    //         ^_____________// current token

    let mut data_types: Vec<DataType> = vec![];
    while matches!(iter.peek(0), &Some(Token::Symbol(_))) {
        let data_type = parse_data_type(iter)?;
        data_types.push(data_type);
    }

    consume_right_paren(iter)?;

    Ok(data_types)
}

fn parse_result_node(iter: &mut PeekableIterator<Token>) -> Result<DataType, CompileError> {
    // (result i32)  //
    //         ^_____// current token
    let data_type = parse_data_type(iter)?;
    consume_right_paren(iter)?;

    Ok(data_type)
}

fn parse_results_node(iter: &mut PeekableIterator<Token>) -> Result<Vec<DataType>, CompileError> {
    // (results i32 i32 i64)  //
    //          ^_____________// current token

    let mut data_types: Vec<DataType> = vec![];
    while matches!(iter.peek(0), &Some(Token::Symbol(_))) {
        let data_type = parse_data_type(iter)?;
        data_types.push(data_type);
    }

    consume_right_paren(iter)?;

    Ok(data_types)
}

fn parse_data_type(iter: &mut PeekableIterator<Token>) -> Result<DataType, CompileError> {
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

fn parse_local_node(iter: &mut PeekableIterator<Token>) -> Result<LocalNode, CompileError> {
    // (local $name i32)  //
    //        ^___________// current token
    let name = maybe_identifier(iter);
    let (memory_data_type, data_length, align) = parse_memory_data_type(iter)?;
    consume_right_paren(iter)?;

    Ok(LocalNode {
        name,
        memory_data_type,
        data_length,
        align,
    })
}

fn parse_locals_node(iter: &mut PeekableIterator<Token>) -> Result<Vec<LocalNode>, CompileError> {
    // (locals i32 i64 f32 f64)  //
    //         ^_________________// current token

    let mut local_nodes: Vec<LocalNode> = vec![];

    while matches!(iter.peek(0), &Some(Token::Symbol(_)))
        || matches!(iter.peek(0), &Some(Token::LeftParen))
    {
        let (memory_data_type, data_length, align) = parse_memory_data_type(iter)?;
        local_nodes.push(LocalNode {
            name: None,
            memory_data_type,
            data_length,
            align,
        });
    }

    consume_right_paren(iter)?;

    Ok(local_nodes)
}

// return '(memory data type, data length, align)'
fn parse_memory_data_type(
    iter: &mut PeekableIterator<Token>,
) -> Result<(MemoryDataType, u32, u16), CompileError> {
    // (local i32)  //
    //        ^_____// current token
    //
    // (local (bytes 12 8))  //
    //        ^______________// current token

    let dt = match iter.next() {
        Some(token) => match token {
            Token::LeftParen => {
                let (data_length, align) = parse_memory_data_type_bytes(iter)?;
                (MemoryDataType::BYTES, data_length, align)
            }
            Token::Symbol(data_type_name) => match data_type_name.as_str() {
                "i32" => (MemoryDataType::I32, 4, 4),
                "i64" => (MemoryDataType::I64, 8, 8),
                "f32" => (MemoryDataType::F32, 4, 4),
                "f64" => (MemoryDataType::F64, 8, 8),
                _ => {
                    return Err(CompileError::new(&format!(
                        "Unknown memory data type: {}",
                        data_type_name
                    )))
                }
            },
            _ => {
                return Err(CompileError::new(&format!(
                    "Expect memory data type, actual: {:?}",
                    token
                )))
            }
        },
        None => return Err(CompileError::new("Missing memory data type")),
    };

    Ok(dt)
}

// return '(data length, align)'
fn parse_memory_data_type_bytes(
    iter: &mut PeekableIterator<Token>,
) -> Result<(u32, u16), CompileError> {
    // (local (bytes 12 8))  //
    //         ^_____________// current token
    //
    // node 'bytes' syntax:
    // '(bytes length align)'

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

    Ok((length, align))
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

fn expect_identifier(
    iter: &mut PeekableIterator<Token>,
    for_what: &str,
) -> Result<String, CompileError> {
    match iter.next() {
        Some(token) => match token {
            Token::Identifier(s) => Ok(s),
            _ => Err(CompileError::new(&format!(
                "Expect an identifier for {}",
                for_what
            ))),
        },
        None => Err(CompileError::new(&format!(
            "Missing an identifier for {}",
            for_what
        ))),
    }
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use ancvm_types::{CompileError, DataType, MemoryDataType};

    use crate::{
        ast::{FuncNode, LocalNode, ModuleElementNode, ModuleNode, ParamNode},
        lexer::lex,
        peekable_iterator::PeekableIterator,
    };

    use super::parse;

    fn parse_from_str(s: &str) -> Result<ModuleNode, CompileError> {
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
            (module "main" (runtime_version "1.0")
                (func $add (param $lhs i32) (param $rhs i64) (result i32) (result i64)
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
                            name: Some("lhs".to_owned()),
                            data_type: DataType::I32
                        },
                        ParamNode {
                            name: Some("rhs".to_owned()),
                            data_type: DataType::I64
                        }
                    ],
                    results: vec![DataType::I32, DataType::I64,],
                    locals: vec![]
                })]
            }
        );

        assert_eq!(
            parse_from_str(
                r#"
            (module "main" (runtime_version "1.0")
                (func $add (params i32 i64 f32 f64) (results i32 f32)
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
                            name: None,
                            data_type: DataType::I32
                        },
                        ParamNode {
                            name: None,
                            data_type: DataType::I64
                        },
                        ParamNode {
                            name: None,
                            data_type: DataType::F32
                        },
                        ParamNode {
                            name: None,
                            data_type: DataType::F64
                        }
                    ],
                    results: vec![DataType::I32, DataType::F32,],
                    locals: vec![]
                })]
            }
        );
    }

    #[test]
    fn test_parse_function_node_local_variables() {
        assert_eq!(
            parse_from_str(
                r#"
            (module "main" (runtime_version "1.0")
                (func $add
                    (local $sum i32) (local $count i64) (local $db (bytes 12 8)) (local $average f32)
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
                            name: Some("sum".to_owned()),
                            memory_data_type: MemoryDataType::I32,
                            data_length: 4,
                            align: 4
                        },
                        LocalNode {
                            name: Some("count".to_owned()),
                            memory_data_type: MemoryDataType::I64,
                            data_length: 8,
                            align: 8
                        },
                        LocalNode {
                            name: Some("db".to_owned()),
                            memory_data_type: MemoryDataType::BYTES,
                            data_length: 12,
                            align: 8
                        },
                        LocalNode {
                            name: Some("average".to_owned()),
                            memory_data_type: MemoryDataType::F32,
                            data_length: 4,
                            align: 4
                        },
                    ]
                })]
            }
        );

        assert_eq!(
            parse_from_str(
                r#"
                (module "main" (runtime_version "1.0")
                (func $add
                    (locals i32 i64 (bytes 12 8) f32)
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
                            name: None,
                            memory_data_type: MemoryDataType::I32,
                            data_length: 4,
                            align: 4
                        },
                        LocalNode {
                            name: None,
                            memory_data_type: MemoryDataType::I64,
                            data_length: 8,
                            align: 8
                        },
                        LocalNode {
                            name: None,
                            memory_data_type: MemoryDataType::BYTES,
                            data_length: 12,
                            align: 8
                        },
                        LocalNode {
                            name: None,
                            memory_data_type: MemoryDataType::F32,
                            data_length: 4,
                            align: 4
                        },
                    ]
                })]
            }
        );
    }
}
