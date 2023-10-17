// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// token types:
//
// - symbol: 'local', 'i32', 'i32_imm', 'user'
// - identifier: '$add'
// - number: '211', '0x11', '0x11_22', '3.14'
// - string: '"..."'
// - line comment: ';; comment'
// - block comment: '(;...;)'

// text examples:
//
// '(local $a i32)'
// '(i32_imm 211)'
// '(i32_imm 0x223) ;; comment'
// '(i32_imm (; also comment ;) 0x11)'
// '(i32_imm (; nest (; comment ;);) 0x11_22)'
// '(import $math (module user "math"))'
// '(import $add (func "add" (param $left i32) (param $right i32) (result i32) (module user "math")))'
// '(import $add (func "add" (type 0) (module $math)))'

use ancvm_types::CompileError;

use crate::peekable_iterator::PeekableIterator;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LeftParen,
    RightParen,
    Identifier(String),
    Number(String),
    String_(String),
    Symbol(String),
}

pub fn lex(iter: &mut PeekableIterator<char>) -> Result<Vec<Token>, CompileError> {
    let mut tokens: Vec<Token> = vec![];

    loop {
        match iter.next() {
            Some(c) => {
                match c {
                    ' ' | '\t' | '\n' | '\r' => {
                        // skip whitespace
                    }
                    '$' => {
                        tokens.push(lex_identifier(c, iter)?);
                    }
                    '0'..='9' => {
                        tokens.push(lex_number(c, iter)?);
                    }
                    '"' => {
                        tokens.push(lex_string(c, iter)?);
                    }
                    '(' => match iter.peek(0) {
                        Some(nc) if *nc == ';' => {
                            comsume_block_comment(iter)?;
                        }
                        _ => tokens.push(Token::LeftParen),
                    },
                    ')' => {
                        tokens.push(Token::RightParen);
                    }
                    ';' if matches!(iter.peek(0), Some(nc) if *nc == ';' ) => {
                        comsume_line_comment(iter)?;
                    }
                    'a'..='z' | 'A'..='Z' | '_' => {
                        tokens.push(lex_symbol(c, iter)?);
                    }
                    _ => return Err(CompileError::new(&format!("Unexpected char: {}", c))),
                }
            }
            None => break,
        }
    }

    Ok(tokens)
}

fn lex_identifier(c: char, iter: &mut PeekableIterator<char>) -> Result<Token, CompileError> {
    todo!()
}

fn lex_number(c: char, iter: &mut PeekableIterator<char>) -> Result<Token, CompileError> {
    todo!()
}

fn lex_string(c: char, iter: &mut PeekableIterator<char>) -> Result<Token, CompileError> {
    todo!()
}

fn lex_symbol(c: char, iter: &mut PeekableIterator<char>) -> Result<Token, CompileError> {
    todo!()
}

fn comsume_block_comment(iter: &mut PeekableIterator<char>) -> Result<(), CompileError> {
    todo!()
}

fn comsume_line_comment(iter: &mut PeekableIterator<char>) -> Result<(), CompileError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Token, peekable_iterator::PeekableIterator};

    use super::lex;

    #[test]
    fn test_lex_empty() {
        let mut chars = "()".chars();
        let mut iter = PeekableIterator::new(&mut chars, 1);
        let result = lex(&mut iter).unwrap();
        assert_eq!(result, vec![Token::LeftParen, Token::RightParen]);
    }
}
