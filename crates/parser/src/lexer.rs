// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// token types:
//
// - identifier:
//   '$' + /a-zA-Z0-9_/+, should not starts with number, e.g.
//   $add, $some_func, $print2
// - symbol:
//   /a-zA-Z0-9_./+, should not starts with number, e.g.
//   local, i32, i32.imm, i32.div_s, user
// - number: supportes decimal, binary, hexadecimal and float point numbers e.g.
//   211, 0x11, 0x11_22, 0b1100, 3.14
// - string: a char sequence surround by double quotes, multiline is supported. e.g.
//   "abc文字😊", "\t\r\n\\\""\u{2d}\u{6587}\0"
//   "line 0
//    line 1"
// - bytes:
//   a char sequence surrounded by char 'b' and double quotes, two hex digital number per byte,
//   separator chars / -\t\r\n/ are allowed, e.g.
//   b"0011aabb", b"00 11 AA BB", b"00-11-aa-bb", b"00 11\nAA BB"
// - line comment: from the double semi-colon to the end of the line, e.g.
//   ;; comment
// - block comment: any block of text surround by '(;' and ';)' pair, nested block comments are allowed, e.g.
//   (; block comment ;)
//   (; one (; two ;);)
//   (; one ;; line comments within the block comment are ignored ;)
//   (; one #( node comments within the block comment are ignored) ;)
// - node comment: a hash mark at the front of the left parenthesis, nested node comments are allowed, e.g.
//   #(add 11 (mul 13 17))
//   #(add 11 #(mul 13 17) (mul 19 23))
//   #(add (; block comments are still valid ;) 11 13)
//   #(add ;; line comments are still valid
//     11 13)
//   #(add (; note ;; line comments within the block comment are ignored ;) 11 13)
//
// ref:
// https://doc.rust-lang.org/reference/tokens.html

// supported escape chars:
//
// - \\, escape char itself
// - \", doube quote
// - \t, horizontal tabulation
// - \r, carriage return (CR)
// - \n, new line character (line feed, LF)
// - \0, null char
// - \u{...}, unicode code point, e.g. '\u{2d}', '\u{6587}'

// ref:
// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#format-control_characters

// unsupported escape chars:
//
// - \v, vertical tabulation
// - \f, page breaking control character, https://en.wikipedia.org/wiki/Page_break#Form_feed
// - \x.., byte escape

// assembly text examples:
//
// '(local $a i32)'
// '(i32.imm 211)'
// '(i32.imm 0x223) ;; comment'
// '(i32.imm (; also comment ;) 0x11)'
// '(i32.imm (; nested (; comment ;);) 0x11_22)'
// '(i32.div_s          ;; multiple lines
//      (i32.imm 11)    ;; left-hand-side
//      #(i32.imm 13)   ;; node comment
//      (i32.imm (;right hand side;) 17)
//  )'
// '(import $math (module (user "math")))'
// '(import $add (module 0) (function "add" (param $left i32) (param $right i32) (result i32)))'
// '(import $add (module $math) (function "add" (type 0)))'

use ancvm_types::CompileError;

use crate::peekable_iterator::PeekableIterator;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LeftParen,
    RightParen,
    Identifier(String),

    // "123", "3.14", "123_456", "0x123abc", "0xaa_bb.11_22", "0b1010.0101"
    Number(String),
    String_(String),
    Bytes(Vec<u8>),
    Symbol(String),
}

pub fn lex(iter: &mut PeekableIterator<char>) -> Result<Vec<Token>, CompileError> {
    let mut tokens: Vec<Token> = vec![];

    while let Some(ch) = iter.next() {
        match ch {
            ' ' | '\t' | '\r' | '\n' => {
                // skip whitespace
            }
            '$' => {
                tokens.push(lex_identifier(iter)?);
            }
            '0'..='9' => {
                tokens.push(lex_number(ch, iter)?);
            }
            'b' if iter.look_ahead_equals(0, &'"') => {
                tokens.push(lex_bytes(iter)?);
            }
            '"' => {
                tokens.push(lex_string(iter)?);
            }
            '(' => {
                if iter.look_ahead_equals(0, &';') {
                    comsume_block_comment(iter)?;
                } else {
                    tokens.push(Token::LeftParen);
                }
            }
            ')' => {
                tokens.push(Token::RightParen);
            }
            ';' => {
                if iter.look_ahead_equals(0, &';') {
                    comsume_line_comment(iter)?;
                } else if iter.look_ahead_equals(0, &')') {
                    return Err(CompileError::new("Unpaired block comment."));
                } else {
                    return Err(CompileError::new("Unexpected char: ;"));
                }
            }
            '#' => {
                if iter.look_ahead_equals(0, &'(') {
                    comsume_node_comment(iter)?;
                } else {
                    return Err(CompileError::new("Unexpected char: #"));
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                tokens.push(lex_symbol(ch, iter)?);
            }
            _ => return Err(CompileError::new(&format!("Unexpected char: {}", ch))),
        }
    }

    Ok(tokens)
}

fn lex_identifier(iter: &mut PeekableIterator<char>) -> Result<Token, CompileError> {
    // $name  //
    //  ^_____// UNverified/current char, i.e. the value of 'iter.peek(0)'

    if matches!(iter.peek(0), Some(nc) if *nc >= '0' && *nc <= '9') {
        // identifier should starts with /a-zA-Z_/
        return Err(CompileError::new(
            "Identifier should not start with a number.",
        ));
    }

    let mut s = String::new();

    while let Some(nc) = iter.peek(0) {
        match *nc {
            '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' => {
                s.push(*nc);
                iter.next();
            }
            ' ' | '\t' | '\r' | '\n' | '(' | ')' | ';' => {
                // terminator chars
                break;
            }
            _ => {
                return Err(CompileError::new(&format!(
                    "Invalid char for identifier: {}",
                    *nc
                )))
            }
        }
    }

    if s.is_empty() {
        Err(CompileError::new("Empty identifier."))
    } else {
        Ok(Token::Identifier(s))
    }
}

fn lex_number(ch: char, iter: &mut PeekableIterator<char>) -> Result<Token, CompileError> {
    // 1234  //
    // ^_____// verified/current char

    if ch == '0' {
        if iter.look_ahead_equals(0, &'b') {
            // '0b...'
            iter.next();
            return lex_number_binary(iter);
        } else if iter.look_ahead_equals(0, &'x') {
            // '0x...'
            iter.next();
            return lex_number_hex(iter);
        }
    }

    lex_number_decimal(ch, iter)
}

fn lex_number_decimal(ch: char, iter: &mut PeekableIterator<char>) -> Result<Token, CompileError> {
    // 1234  //
    // ^_____// verified/current char

    let mut s = String::new();
    s.push(ch);

    while let Some(nc) = iter.peek(0) {
        match *nc {
            '0'..='9' | '_' | '.' => {
                s.push(*nc);
                iter.next();
            }
            ' ' | '\t' | '\r' | '\n' | '(' | ')' | ';' => {
                // terminator chars
                break;
            }
            _ => {
                return Err(CompileError::new(&format!(
                    "Invalid char for decimal number: {}",
                    *nc
                )))
            }
        }
    }

    Ok(Token::Number(s))
}

fn lex_number_binary(iter: &mut PeekableIterator<char>) -> Result<Token, CompileError> {
    // 0b0101  //
    //   ^_____// UNverified/current char

    let mut s = String::new();
    s.push_str("0b");

    while let Some(nc) = iter.peek(0) {
        match *nc {
            '0' | '1' | '_' | '.' => {
                s.push(*nc);
                iter.next();
            }
            ' ' | '\t' | '\r' | '\n' | '(' | ')' | ';' => {
                // terminator chars
                break;
            }
            _ => {
                return Err(CompileError::new(&format!(
                    "Invalid char for binary number: {}",
                    *nc
                )))
            }
        }
    }

    if s.len() < 3 {
        Err(CompileError::new("Incomplete binary number"))
    } else {
        Ok(Token::Number(s))
    }
}

fn lex_number_hex(iter: &mut PeekableIterator<char>) -> Result<Token, CompileError> {
    // 0xabcd  //
    //   ^_____// UNverified/current char

    let mut s = String::new();
    s.push_str("0x");

    while let Some(nc) = iter.peek(0) {
        match *nc {
            '0'..='9' | 'a'..='f' | 'A'..='F' | '_' | '.' => {
                s.push(*nc);
                iter.next();
            }
            ' ' | '\t' | '\r' | '\n' | '(' | ')' | ';' => {
                // terminator chars
                break;
            }
            _ => {
                return Err(CompileError::new(&format!(
                    "Invalid char for hexadecimal number: {}",
                    *nc
                )))
            }
        }
    }

    if s.len() < 3 {
        Err(CompileError::new("Incomplete hex number"))
    } else {
        Ok(Token::Number(s))
    }
}

fn lex_string(iter: &mut PeekableIterator<char>) -> Result<Token, CompileError> {
    // "abc"  //
    //  ^_____// UNverified/current char

    let mut s = String::new();

    loop {
        match iter.next() {
            Some(ch) => match ch {
                '\\' => {
                    // escape char
                    match iter.peek(0) {
                        Some(nc) => {
                            match *nc {
                                '\\' => {
                                    s.push('\\');
                                    iter.next();
                                }
                                '"' => {
                                    s.push('"');
                                    iter.next();
                                }
                                't' => {
                                    // horizontal tabulation
                                    s.push('\t');
                                    iter.next();
                                }
                                'r' => {
                                    // carriage return (CR)
                                    s.push('\r');
                                    iter.next();
                                }
                                'n' => {
                                    // new line character (line feed, LF)
                                    s.push('\n');
                                    iter.next();
                                }
                                '0' => {
                                    // null char
                                    s.push('\0');
                                    iter.next();
                                }
                                'u' => {
                                    // unicode code point, e.g. '\u{2d}', '\u{6587}'
                                    iter.next();
                                    s.push(lex_string_unescape_unicode(iter)?);
                                }
                                _ => {
                                    return Err(CompileError::new(&format!(
                                        "Unsupported escape char for string: {}",
                                        nc
                                    )))
                                }
                            }
                        }
                        None => {
                            return Err(CompileError::new("Incomplete escape char for string."))
                        }
                    }
                }
                '"' => {
                    // end of the string
                    break;
                }
                _ => {
                    // ordinary char
                    s.push(ch);
                }
            },
            None => return Err(CompileError::new("Missing end quote for string.")),
        }
    }

    Ok(Token::String_(s))
}

fn lex_string_unescape_unicode(iter: &mut PeekableIterator<char>) -> Result<char, CompileError> {
    // \u{6587}  //
    //   ^_______// UNverified/current char

    if !matches!(iter.next(), Some(c) if c == '{') {
        return Err(CompileError::new(
            "Missing left brace for unicode escape sequence.",
        ));
    }

    // \u{6587}  //
    //    ^______// UNverified/current char

    let mut s = String::new();

    loop {
        match iter.next() {
            Some(ch) => match ch {
                '}' => break,
                '0'..='9' | 'a'..='f' | 'A'..='F' => s.push(ch),
                _ => {
                    return Err(CompileError::new(&format!(
                        "Invalid character for unicode escape sequence: {}",
                        ch
                    )))
                }
            },
            None => {
                return Err(CompileError::new(
                    "Missing right brace for unicode escape sequence.",
                ))
            }
        }

        if s.len() > 5 {
            return Err(CompileError::new(
                "The value of unicode point code is to large.",
            ));
        }
    }

    let code_point = u32::from_str_radix(&s, 16).unwrap();

    if let Some(c) = char::from_u32(code_point) {
        Ok(c)
    } else {
        Err(CompileError::new("Invalid unicode code point."))
    }
}

fn lex_bytes(iter: &mut PeekableIterator<char>) -> Result<Token, CompileError> {
    // b"0011aabb"  //
    //  ^___________// verified/current char

    let mut bytes: Vec<u8> = Vec::new();
    let mut buf = String::new();

    iter.next(); // consume the quote '"'

    loop {
        match iter.next() {
            Some(ch) => {
                match ch {
                    ' ' | '\t' | '\r' | '\n' | '-' => {
                        // ignore
                    }
                    '"' => {
                        if !buf.is_empty() {
                            return Err(CompileError::new("Incomplete byte string."));
                        } else {
                            break;
                        }
                    }
                    'a'..='f' | 'A'..='F' | '0'..='9' => {
                        buf.push(ch);

                        if buf.len() == 2 {
                            let b = u8::from_str_radix(&buf, 16).unwrap();
                            bytes.push(b);
                            buf.clear();
                        }
                    }
                    _ => {
                        return Err(CompileError::new(&format!(
                            "Invalid char for byte string: {}",
                            ch
                        )))
                    }
                }
            }
            None => return Err(CompileError::new("Missing end quote for byte string.")),
        }
    }

    Ok(Token::Bytes(bytes))
}

fn comsume_line_comment(iter: &mut PeekableIterator<char>) -> Result<(), CompileError> {
    // ;;...  //
    // ^^_____// verified
    // |______// current char

    iter.next(); // consume the char ';'

    while let Some(c) = iter.next() {
        if c == '\n' {
            break;
        }
    }

    Ok(())
}

fn comsume_block_comment(iter: &mut PeekableIterator<char>) -> Result<(), CompileError> {
    // (;...;)  //
    // ^^_______// verified
    // |________// current char

    iter.next(); // consume the char ';'

    let mut pairs = 1;

    loop {
        match iter.next() {
            Some(ch) => match ch {
                '(' if iter.look_ahead_equals(0, &';') => {
                    iter.next();
                    pairs += 1;
                }
                ';' if iter.look_ahead_equals(0, &')') => {
                    iter.next();
                    pairs -= 1;

                    // check pairs
                    if pairs == 0 {
                        break;
                    }
                }
                _ => {
                    // ignore
                }
            },
            None => return Err(CompileError::new("Incomplete block comment.")),
        }
    }

    Ok(())
}

fn comsume_node_comment(iter: &mut PeekableIterator<char>) -> Result<(), CompileError> {
    // #(comment ...)  //
    // ^^______________// verified
    // |_______________// current char

    iter.next(); // consume the char '('

    let mut pairs = 1;

    loop {
        match iter.next() {
            Some(ch) => match ch {
                '(' => {
                    if iter.look_ahead_equals(0, &';') {
                        comsume_block_comment(iter)?;
                    } else {
                        pairs += 1;
                    }
                }
                ')' => {
                    pairs -= 1;

                    if pairs == 0 {
                        break;
                    }
                }
                ';' => {
                    if iter.look_ahead_equals(0, &';') {
                        comsume_line_comment(iter)?;
                    } else if iter.look_ahead_equals(0, &')') {
                        return Err(CompileError::new("Unpaired block comment."));
                    } else {
                        return Err(CompileError::new("Unexpected char: ;"));
                    }
                }
                _ => {
                    // continue
                }
            },
            None => return Err(CompileError::new("Incomplete node comment.")),
        }
    }

    Ok(())
}

fn lex_symbol(ch: char, iter: &mut PeekableIterator<char>) -> Result<Token, CompileError> {
    // i32.imm  //
    // ^________// verified/current char

    let mut s = String::new();
    s.push(ch);

    while let Some(nc) = iter.peek(0) {
        match *nc {
            '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' | '.' => {
                s.push(*nc);
                iter.next();
            }
            ' ' | '\t' | '\r' | '\n' | '(' | ')' | ';' => {
                // terminator chars
                break;
            }
            _ => {
                return Err(CompileError::new(&format!(
                    "Invalid char for symbol: {}",
                    *nc
                )))
            }
        }
    }

    Ok(Token::Symbol(s))
}

impl Token {
    pub fn new_identifier(s: &str) -> Self {
        Token::Identifier(s.to_owned())
    }

    pub fn new_number(s: &str) -> Self {
        Token::Number(s.to_owned())
    }

    pub fn new_string(s: &str) -> Self {
        Token::String_(s.to_owned())
    }

    pub fn new_bytes(slice: &[u8]) -> Self {
        Token::Bytes(slice.to_vec())
    }

    pub fn new_symbol(s: &str) -> Self {
        Token::Symbol(s.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use ancvm_types::CompileError;

    use crate::{lexer::Token, peekable_iterator::PeekableIterator};

    use super::lex;

    fn lex_from_str(s: &str) -> Result<Vec<Token>, CompileError> {
        let mut chars = s.chars();
        let mut iter = PeekableIterator::new(&mut chars, 1);
        lex(&mut iter)
    }

    #[test]
    fn test_lex_white_spaces() {
        assert_eq!(lex_from_str("  ").unwrap(), vec![]);

        assert_eq!(
            lex_from_str("()").unwrap(),
            vec![Token::LeftParen, Token::RightParen]
        );

        assert_eq!(
            lex_from_str("(  )").unwrap(),
            vec![Token::LeftParen, Token::RightParen]
        );

        assert_eq!(
            lex_from_str("(\t\r\n)").unwrap(),
            vec![Token::LeftParen, Token::RightParen]
        );
    }

    #[test]
    fn test_lex_identifier() {
        assert_eq!(
            lex_from_str("$name").unwrap(),
            vec![Token::new_identifier("name")]
        );

        assert_eq!(
            lex_from_str("($name)").unwrap(),
            vec![
                Token::LeftParen,
                Token::new_identifier("name"),
                Token::RightParen
            ]
        );

        assert_eq!(
            lex_from_str("( $a )").unwrap(),
            vec![
                Token::LeftParen,
                Token::new_identifier("a"),
                Token::RightParen
            ]
        );

        assert_eq!(
            lex_from_str("$foo $bar").unwrap(),
            vec![Token::new_identifier("foo"), Token::new_identifier("bar"),]
        );

        // incomplete identifier
        assert!(matches!(
            lex_from_str("$"),
            Err(CompileError { message: _ })
        ));

        // invalid identifier
        assert!(matches!(
            lex_from_str("$1abc"),
            Err(CompileError { message: _ })
        ));

        // invalid char for identifier
        assert!(matches!(
            lex_from_str("$abc+xyz"),
            Err(CompileError { message: _ })
        ));
    }

    #[test]
    fn test_lex_number() {
        assert_eq!(lex_from_str("123").unwrap(), vec![Token::new_number("123")]);

        assert_eq!(
            lex_from_str("(123)").unwrap(),
            vec![
                Token::LeftParen,
                Token::new_number("123"),
                Token::RightParen
            ]
        );

        assert_eq!(
            lex_from_str("123.456").unwrap(),
            vec![Token::new_number("123.456")]
        );

        assert_eq!(
            lex_from_str("12_34_56").unwrap(),
            vec![Token::new_number("12_34_56")]
        );

        assert_eq!(
            lex_from_str("123 456").unwrap(),
            vec![Token::new_number("123"), Token::new_number("456")]
        );

        assert_eq!(
            lex_from_str("0x1234abcd").unwrap(),
            vec![Token::new_number("0x1234abcd")]
        );

        assert_eq!(
            lex_from_str("0xee_ff.1122").unwrap(),
            vec![Token::new_number("0xee_ff.1122")]
        );

        assert_eq!(
            lex_from_str("0b00110101").unwrap(),
            vec![Token::new_number("0b00110101")]
        );

        assert_eq!(
            lex_from_str("0b00_11.0101").unwrap(),
            vec![Token::new_number("0b00_11.0101")]
        );

        // invalid char for decimal number
        assert!(matches!(
            lex_from_str("123abc"),
            Err(CompileError { message: _ })
        ));

        // invalid char for decimal number
        assert!(matches!(
            lex_from_str("123-456"),
            Err(CompileError { message: _ })
        ));

        // incomplete hex number
        assert!(matches!(
            lex_from_str("0x"),
            Err(CompileError { message: _ })
        ));

        // invalid char for hex number
        assert!(matches!(
            lex_from_str("0x123xyz"),
            Err(CompileError { message: _ })
        ));

        // incomplete binary number
        assert!(matches!(
            lex_from_str("0b"),
            Err(CompileError { message: _ })
        ));

        // invalid char for binary number
        assert!(matches!(
            lex_from_str("0b1234"),
            Err(CompileError { message: _ })
        ));
    }

    #[test]
    fn test_lex_string() {
        assert_eq!(lex_from_str("\"\"").unwrap(), vec![Token::new_string("")]);

        assert_eq!(
            lex_from_str("\"abc\"").unwrap(),
            vec![Token::new_string("abc")]
        );

        assert_eq!(
            lex_from_str("(\"abc\")").unwrap(),
            vec![
                Token::LeftParen,
                Token::new_string("abc"),
                Token::RightParen
            ]
        );

        assert_eq!(
            lex_from_str("\"abc\" \"xyz\"").unwrap(),
            vec![Token::new_string("abc"), Token::new_string("xyz"),]
        );

        assert_eq!(
            lex_from_str("\"abc\"\n\n\"xyz\"").unwrap(),
            vec![Token::new_string("abc"), Token::new_string("xyz"),]
        );

        assert_eq!(
            lex_from_str(
                r#"
            "abc文字😊"
            "#
            )
            .unwrap(),
            vec![Token::new_string("abc文字😊")]
        );

        assert_eq!(
            lex_from_str(
                r#"
            "\r\n\t\\\"\u{2d}\u{6587}\0"
            "#
            )
            .unwrap(),
            vec![Token::new_string("\r\n\t\\\"-文\0")]
        );

        // unsupported escape char \v
        assert!(matches!(
            lex_from_str(
                r#"
            "abc\vxyz"
            "#
            ),
            Err(CompileError { message: _ })
        ));

        // unsupported byte escape \x..
        assert!(matches!(
            lex_from_str(
                r#"
            "abc\x33xyz"
            "#
            ),
            Err(CompileError { message: _ })
        ));

        // incomplete escape string
        assert!(matches!(
            lex_from_str(r#""abc\"#),
            Err(CompileError { message: _ })
        ));

        // unicode code point is too large
        assert!(matches!(
            lex_from_str(
                r#"
            "abc\u{123456}xyz"
            "#
            ),
            Err(CompileError { message: _ })
        ));

        // invalid char for unicode escape sequence
        assert!(matches!(
            lex_from_str(
                r#"
            "abc\u{mn}xyz"
            "#
            ),
            Err(CompileError { message: _ })
        ));

        // missing left brace for unicode escape sequence
        assert!(matches!(
            lex_from_str(
                r#"
            "abc\u1234}xyz"
            "#
            ),
            Err(CompileError { message: _ })
        ));

        // missing right brace for unicode escape sequence
        assert!(matches!(
            lex_from_str(r#""abc\u{1234"#),
            Err(CompileError { message: _ })
        ));

        // missing right quote
        assert!(matches!(
            lex_from_str(
                r#"
            "abc
            "#
            ),
            Err(CompileError { message: _ })
        ));
    }

    #[test]
    fn test_lex_bytes() {
        assert_eq!(
            lex_from_str(
                r#"
            b""
            "#
            )
            .unwrap(),
            vec![Token::Bytes(vec![])]
        );

        assert_eq!(
            lex_from_str(
                r#"
            b"11131719"
            "#
            )
            .unwrap(),
            vec![Token::Bytes(vec![0x11, 0x13, 0x17, 0x19])]
        );

        assert_eq!(
            lex_from_str(
                r#"
            b"11 13 1719"
            "#
            )
            .unwrap(),
            vec![Token::Bytes(vec![0x11, 0x13, 0x17, 0x19])]
        );

        assert_eq!(
            lex_from_str(
                r#"
            b"11-13-1719"
            "#
            )
            .unwrap(),
            vec![Token::Bytes(vec![0x11, 0x13, 0x17, 0x19])]
        );

        assert_eq!(
            lex_from_str(
                "
            b\"1113\n17\t19\"
            "
            )
            .unwrap(),
            vec![Token::Bytes(vec![0x11, 0x13, 0x17, 0x19])]
        );

        // incomplete byte string
        assert!(matches!(
            lex_from_str(
                r#"
            b"1113171"
            "#
            ),
            Err(CompileError { message: _ })
        ));

        // invalid char for byte string
        assert!(matches!(
            lex_from_str(
                r#"
            b"1113171z"
            "#
            ),
            Err(CompileError { message: _ })
        ));

        // missing end quote
        assert!(matches!(
            lex_from_str(
                r#"
            b"11131719
            "#
            ),
            Err(CompileError { message: _ })
        ));
    }

    #[test]
    fn test_lex_line_comment() {
        assert_eq!(
            lex_from_str(
                r#"
            7 ;;11
            13 17;; 19 23
            ;; 29
            31;; 37
            "#
            )
            .unwrap(),
            vec![
                Token::new_number("7"),
                Token::new_number("13"),
                Token::new_number("17"),
                Token::new_number("31"),
            ]
        );
    }

    #[test]
    fn test_lex_block_comment() {
        assert_eq!(
            lex_from_str(
                r#"
            7 (; 11 13 ;) 17
            "#
            )
            .unwrap(),
            vec![Token::new_number("7"), Token::new_number("17"),]
        );

        assert_eq!(
            lex_from_str(
                r#"
            7 (; 11 (; 13 ;) 17 ;) 19
            "#
            )
            .unwrap(),
            vec![Token::new_number("7"), Token::new_number("19"),]
        );

        assert_eq!(
            lex_from_str(
                r#"
            7 (; 11 ;; 13 17 ;) 19
            "#
            )
            .unwrap(),
            vec![Token::new_number("7"), Token::new_number("19"),]
        );

        assert_eq!(
            lex_from_str(
                r#"
            7 (; 11 #(13 17) ;) 19
            "#
            )
            .unwrap(),
            vec![Token::new_number("7"), Token::new_number("19"),]
        );

        // missing end pair
        assert!(matches!(
            lex_from_str(
                r#"
            7 (; 11 (; 13 ;) 17
            "#
            ),
            Err(CompileError { message: _ })
        ));

        // unpaired
        assert!(matches!(
            lex_from_str(
                r#"
            7 ;) 11
            "#
            ),
            Err(CompileError { message: _ })
        ));
    }

    #[test]
    fn test_lex_node_comment() {
        assert_eq!(
            lex_from_str(
                r#"
            7 #(add 11 (mul 13 17)) 29
            "#
            )
            .unwrap(),
            vec![Token::new_number("7"), Token::new_number("29"),]
        );

        assert_eq!(
            lex_from_str(
                r#"
            7 #(add 11 #(mul 13 17) (mul 19 23)) 29
            "#
            )
            .unwrap(),
            vec![Token::new_number("7"), Token::new_number("29"),]
        );

        assert_eq!(
            lex_from_str(
                r#"
            7 #(add (; 11 ;)) 29
            "#
            )
            .unwrap(),
            vec![Token::new_number("7"), Token::new_number("29"),]
        );

        assert_eq!(
            lex_from_str(
                r#"
            7 #(add ;; 11 13)
            ) 29
            "#
            )
            .unwrap(),
            vec![Token::new_number("7"), Token::new_number("29"),]
        );

        assert_eq!(
            lex_from_str(
                r#"
            7 #(add (; 11 ;; 13 ;)) 29
            "#
            )
            .unwrap(),
            vec![Token::new_number("7"), Token::new_number("29"),]
        );

        // missing end pair
        assert!(matches!(
            lex_from_str(
                r#"
            7 #( 11
            "#
            ),
            Err(CompileError { message: _ })
        ));

        // missing end pair
        assert!(matches!(
            lex_from_str(
                r#"
            7 #) 11
            "#
            ),
            Err(CompileError { message: _ })
        ));

        // missing end pair
        assert!(matches!(
            lex_from_str(
                r#"
            7 #( 11 ()
            "#
            ),
            Err(CompileError { message: _ })
        ));
    }

    #[test]
    fn test_lex_symbol() {
        assert_eq!(
            lex_from_str("name").unwrap(),
            vec![Token::new_symbol("name")]
        );

        assert_eq!(
            lex_from_str("i32.imm").unwrap(),
            vec![Token::new_symbol("i32.imm")]
        );

        assert_eq!(
            lex_from_str("i32.div_s").unwrap(),
            vec![Token::new_symbol("i32.div_s")]
        );

        assert_eq!(
            lex_from_str("(name)").unwrap(),
            vec![
                Token::LeftParen,
                Token::new_symbol("name"),
                Token::RightParen
            ]
        );

        assert_eq!(
            lex_from_str("( a )").unwrap(),
            vec![Token::LeftParen, Token::new_symbol("a"), Token::RightParen]
        );

        assert_eq!(
            lex_from_str("foo bar").unwrap(),
            vec![Token::new_symbol("foo"), Token::new_symbol("bar"),]
        );

        // invalid symbol
        assert!(matches!(
            lex_from_str("1abc"),
            Err(CompileError { message: _ })
        ));

        // invalid char for symbol
        assert!(matches!(
            lex_from_str("abc+xyz"),
            Err(CompileError { message: _ })
        ));
    }

    #[test]
    fn test_lex_text() {
        assert_eq!(
            lex_from_str(
                r#"
            (local $a i32)
            "#
            )
            .unwrap(),
            vec![
                Token::LeftParen,
                Token::new_symbol("local"),
                Token::new_identifier("a"),
                Token::new_symbol("i32"),
                Token::RightParen,
            ]
        );

        assert_eq!(
            lex_from_str(
                r#"
            (i32.imm 211)
            "#
            )
            .unwrap(),
            vec![
                Token::LeftParen,
                Token::new_symbol("i32.imm"),
                Token::new_number("211"),
                Token::RightParen,
            ]
        );

        assert_eq!(
            lex_from_str(
                r#"
            (i32.imm 0x223) ;; comment
            "#
            )
            .unwrap(),
            vec![
                Token::LeftParen,
                Token::new_symbol("i32.imm"),
                Token::new_number("0x223"),
                Token::RightParen,
            ]
        );

        assert_eq!(
            lex_from_str(
                r#"
            (i32.imm (; also comment ;) 0x11)
            "#
            )
            .unwrap(),
            vec![
                Token::LeftParen,
                Token::new_symbol("i32.imm"),
                Token::new_number("0x11"),
                Token::RightParen,
            ]
        );

        assert_eq!(
            lex_from_str(
                r#"
            (i32.imm (; nest (; comment ;);) 0x11_22)
            "#
            )
            .unwrap(),
            vec![
                Token::LeftParen,
                Token::new_symbol("i32.imm"),
                Token::new_number("0x11_22"),
                Token::RightParen,
            ]
        );

        assert_eq!(
            lex_from_str(
                r#"
            (i32.div_s          ;; multiple lines
                (i32.imm 11)    ;; left-hand-side
                #(i32.imm 13)   ;; node comment
                (i32.imm (;right hand side;) 17)
            )
            "#
            )
            .unwrap(),
            vec![
                Token::LeftParen,
                Token::new_symbol("i32.div_s"),
                Token::LeftParen,
                Token::new_symbol("i32.imm"),
                Token::new_number("11"),
                Token::RightParen,
                Token::LeftParen,
                Token::new_symbol("i32.imm"),
                Token::new_number("17"),
                Token::RightParen,
                Token::RightParen,
            ]
        );

        assert_eq!(
            lex_from_str(
                r#"
            (import $math (module (user "math")))
            "#
            )
            .unwrap(),
            vec![
                Token::LeftParen,
                Token::new_symbol("import"),
                Token::new_identifier("math"),
                Token::LeftParen,
                Token::new_symbol("module"),
                Token::LeftParen,
                Token::new_symbol("user"),
                Token::new_string("math"),
                Token::RightParen,
                Token::RightParen,
                Token::RightParen,
            ]
        );

        assert_eq!(
            lex_from_str(
                r#"
            (import $add (module $math) (function "add" (type 0)))
            "#
            )
            .unwrap(),
            vec![
                Token::LeftParen,
                Token::new_symbol("import"),
                Token::new_identifier("add"),
                Token::LeftParen,
                Token::new_symbol("module"),
                Token::new_identifier("math"),
                Token::RightParen,
                Token::LeftParen,
                Token::new_symbol("function"),
                Token::new_string("add"),
                Token::LeftParen,
                Token::new_symbol("type"),
                Token::new_number("0"),
                Token::RightParen,
                Token::RightParen,
                Token::RightParen,
            ]
        );
    }
}
