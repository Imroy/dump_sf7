/*
  sega-basic
  Copyright 2023 Ian Tester

  This library is free software: you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.

  This library is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.

  You should have received a copy of the GNU General Public License
  along with this library.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Sega BASIC routines
//!
//! ## Line format
//!
//! Each line follows this format:
//! - Content length (one byte)
//! - Line number (two bytes, little endian order)
//! - Two unknown bytes that are always zero
//! - Content
//! - 0x0d as newline character
//!
//! ### Content format
//! - Content starts with a a 'command' byte code after optional ASCII characters e.g a space
//! - The following bytes are any number of ASCII text or 'function' byte codes.
//! The function codes seem to overlay the command ones, so we fallback to trying them if no function is found.
//! - Command and function codes have their high bit set, so they can be easily distinguished
//! from regular ASCII text.
//! - After a colon (':') another command is given and the format restarts.
//! - REMarks consume the rest of the line with 8-bit text.
//! - Quoted text strings and REMarks can use the whole 8-bit [character set](sc3000_charset).
//!
//! ## Commands
//!
//! |    | x0 | x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8 | x9 | xA | xB | xC | xD | xE | xF |
//! |----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
//! | **8x** | | INPUT$ | LIST | LLIST | AUTO | DELETE | RUN | CONT | LOAD | SAVE | VERIFY | NEW | RENUM | FILES | LFILES | BOOT |
//! | **9x** | REM | PRINT | LPRINT | DATA | DEF | INPUT | READ | STOP | END | LET | DIM | FOR | NEXT | GOTO | GOSUB | GO |
//! | **Ax** | ON | RETURN | ERASE | CURSOR | IF | RESTORE | SCREEN | COLOR | LINE | SOUND | BEEP | CONSOLE | CLS | OUT | CALL | POKE |
//! | **Bx** | PSET | PRESET | PAINT | BLINE | POSITION | HCOPY | SPRITE | PATTERN | CIRCLE | BCIRCLE | MAG | VPOKE | MOTOR | OPEN | CLOSE | COMSET |
//! | **Cx** | ^ | * | / | MOD | + | - | <> | >= | <= | > | < | = | NOT | AND | OR | XOR |
//! | **Dx** | CLOADM | CSAVEM | VERIFYM |  |  |  |  |  |  |  |  |  |  |  |  |  |
//! | **Ex** | FN | TO | STEP | THEN | TAB | SPC | OUTPUT |  |  |  |  |  |  |  |  |  |
//!
//! ## Functions
//!
//! |    | x0 | x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8 | x9 | xA | xB | xC | xD | xE | xF |
//! |----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
//! | **8x** | ABS| RND | SIN | COS | TAN | ASN | ACS | ATN | LOG | LGT | LTW | EXP | RAD | DEG | PI | SQR |
//! | **9x** | INT | SGN | ASC | LEN | VAL | PEEK | INP | FRE | VPEEK | STICK | STRIG | EOF | LOC | LOD | DSKF | |
//! | **Ax** | CHR$ | HEX$ | INKEY$ | LEFT$ | RIGHT$ | MID$ | STR$ | TIME$ | | | | | | | | |


#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use sc3000_charset::{CharacterSet, SC3000String};


static COMMANDLIST: [(u8, &'static str); 89] = [
    ( 0x81, "INPUT$" ), ( 0x82, "LIST" ), ( 0x83, "LLIST" ),
    ( 0x84, "AUTO" ), ( 0x85, "DELETE" ), ( 0x86, "RUN" ), ( 0x87, "CONT" ),
    ( 0x88, "LOAD" ), ( 0x89, "SAVE" ), ( 0x8a, "VERIFY" ), ( 0x8b, "NEW" ),
    ( 0x8c, "RENUM" ), ( 0x8d, "FILES" ), ( 0x8e, "LFILES" ), ( 0x8f, "BOOT" ),

    ( 0x90, "REM" ), ( 0x91, "PRINT" ), ( 0x92, "LPRINT" ), ( 0x93, "DATA" ),
    ( 0x94, "DEF" ), ( 0x95, "INPUT" ), ( 0x96, "READ" ), ( 0x97, "STOP" ),
    ( 0x98, "END" ), ( 0x99, "LET" ), ( 0x9a, "DIM" ), ( 0x9b, "FOR" ),
    ( 0x9c, "NEXT" ), ( 0x9d, "GOTO" ), ( 0x9e, "GOSUB" ), ( 0x9f, "GO" ),

    ( 0xa0, "ON" ), ( 0xa1, "RETURN" ), ( 0xa2, "ERASE" ), ( 0xa3, "CURSOR" ),
    ( 0xa4, "IF" ), ( 0xa5, "RESTORE" ), ( 0xa6, "SCREEN" ), ( 0xa7, "COLOR" ),
    ( 0xa8, "LINE" ), ( 0xa9, "SOUND" ), ( 0xaa, "BEEP" ), ( 0xab, "CONSOLE" ),
    ( 0xac, "CLS" ), ( 0xad, "OUT" ), ( 0xae, "CALL" ), ( 0xaf, "POKE" ),

    ( 0xb0, "PSET" ), ( 0xb1, "PRESET" ), ( 0xb2, "PAINT" ), ( 0xb3, "BLINE" ),
    ( 0xb4, "POSITION" ), ( 0xb5, "HCOPY" ), ( 0xb6, "SPRITE" ), ( 0xb7, "PATTERN" ),
    ( 0xb8, "CIRCLE" ), ( 0xb9, "BCIRCLE" ), ( 0xba, "MAG" ), ( 0xbb, "VPOKE" ),
    ( 0xbc, "MOTOR" ), ( 0xbd, "OPEN" ), ( 0xbe, "CLOSE" ), ( 0xbf, "COMSET" ),

    ( 0xc0, "^" ), ( 0xc1, "*" ), ( 0xc2, "/" ), ( 0xc3, "MOD" ),
    ( 0xc4, "+" ), ( 0xc5, "-" ), ( 0xc6, "<>" ), ( 0xc7, ">=" ),
    ( 0xc8, "<=" ), ( 0xc9, ">" ), ( 0xca, "<" ), ( 0xcb, "=" ),
    ( 0xcc, "NOT" ), ( 0xcd, "AND" ), ( 0xce, "OR" ), ( 0xcf, "XOR" ),

    ( 0xd0, "CLOADM" ), ( 0xd1, "CSAVEM" ), ( 0xd2, "VERIFYM" ),

    ( 0xe0, "FN" ), ( 0xe1, "TO" ), ( 0xe2, "STEP" ), ( 0xe3, "THEN" ),
    ( 0xe4, "TAB" ), ( 0xe5, "SPC" ), ( 0xe7, "OUTPUT" ),
];

static FUNCLIST: [(u8, &'static str); 39] = [
    ( 0x80, "ABS" ), ( 0x81, "RND" ), ( 0x82, "SIN" ), ( 0x83, "COS" ),
    ( 0x84, "TAN" ), ( 0x85, "ASN" ), ( 0x86, "ACS" ), ( 0x87, "ATN" ),
    ( 0x88, "LOG" ), ( 0x89, "LGT" ), ( 0x8a, "LTW" ), ( 0x8b, "EXP" ),
    ( 0x8c, "RAD" ), ( 0x8d, "DEG" ), ( 0x8e, "PI" ), ( 0x8f, "SQR" ),

    ( 0x90, "INT" ), ( 0x91, "SGN" ), ( 0x92, "ASC" ), ( 0x93, "LEN" ),
    ( 0x94, "VAL" ), ( 0x95, "PEEK" ), ( 0x96, "INP" ), ( 0x97, "FRE" ),
    ( 0x98, "VPEEK" ), ( 0x99, "STICK" ), ( 0x9a, "STRIG" ), ( 0x9b, "EOF" ),
    ( 0x9c, "LOC" ), ( 0x9d, "LOD" ), ( 0x9e, "DSKF" ),

    ( 0xa0, "CHR$" ), ( 0xa1, "HEX$" ), ( 0xa2, "INKEY$" ), ( 0xa3, "LEFT$" ),
    ( 0xa4, "RIGHT$" ), ( 0xa5, "MID$" ), ( 0xa6, "STR$" ), ( 0xa7, "TIME$" ),
];

lazy_static! {
    static ref COMMANDS: HashMap<u8, &'static str> = HashMap::from(COMMANDLIST);
    static ref FUNCS: HashMap<u8, &'static str> = HashMap::from(FUNCLIST);
}

#[derive(Copy, Clone, Debug)]
enum TokenState {
    Command,
    Remark,
    ASCIIAndFuncs,
    QuotedString,
}

fn detokenise_line(output: &mut String, line: &[u8], cset: CharacterSet) {
    let mut temp_bytes = vec![];

    // We don't use the content length in the first byte

    // Next two bytes are the line number
    let lineno = (line[1] as u16) | ((line[2] as u16) << 8);

    // Next two bytes?

    // Print the line number
    output.push_str(&lineno.to_string());
    output.push(' ');

    let mut state = TokenState::Command;
    let mut j = 5;
    while j < line.len() {
        let b = line[j];
        match state {
            TokenState::Command => {
                // Ordinary ASCII character
                if b < 128 {
                    temp_bytes.push(b);
                    if b == b'"' {
                        state = TokenState::QuotedString;
                    }

                    j += 1;
                    continue;
                }

                if let Some(cmdname) = COMMANDS.get(&b) {
                    if !temp_bytes.is_empty() {
                        output.push_str(&SC3000String::from_cset(temp_bytes.as_slice(), cset).to_string());
                        temp_bytes.clear();
                    }
                    output.push_str(cmdname);

                    // REM
                    if b == 0x90 {
                        state = TokenState::Remark;
                    } else {
                        state = TokenState::ASCIIAndFuncs;
                    };

                    j += 1;
                    continue;
                }
            },

            TokenState::Remark => {
                // Any 8-bit character
                temp_bytes.push(b);

                j += 1;
                continue;
            },

            TokenState::ASCIIAndFuncs => {
                // Ordinary ASCII character
                if b < 128 {
                    temp_bytes.push(b);
                    if b == b':' {
                        state = TokenState::Command;
                    } else if b == b'"' {
                        state = TokenState::QuotedString;
                    }

                    j += 1;
                    continue;
                }

                if let Some(funcname) = FUNCS.get(&b) {
                    if !temp_bytes.is_empty() {
                        output.push_str(&SC3000String::from_cset(temp_bytes.as_slice(), cset).to_string());
                        temp_bytes.clear();
                    }
                    output.push_str(funcname);

                    j += 1;
                    continue;
                }

                // fallback to commands
                if let Some(cmdname) = COMMANDS.get(&b) {
                    if !temp_bytes.is_empty() {
                        output.push_str(&SC3000String::from_cset(temp_bytes.as_slice(), cset).to_string());
                        temp_bytes.clear();
                    }
                    output.push_str(cmdname);

                    j += 1;
                    continue;
                }
            },

            TokenState::QuotedString => {
                // Any 8-bit character
                temp_bytes.push(b);

                // End the quoted string
                if b == b'"' {
                    state = TokenState::ASCIIAndFuncs;
                }

                j += 1;
                continue;
            },
        }

        // ?
        j += 1;
    }

    if !temp_bytes.is_empty() {
        output.push_str(&SC3000String::from_cset(temp_bytes.as_slice(), cset).to_string());
    }
}

/// Detokenise a byte slice of data holding BASIC source code into a Unicode string
pub fn detokenise(sc3kstr: &SC3000String) -> String {
    let mut output = String::with_capacity(sc3kstr.len() * 10);

    let mut i = 0;
    while i < sc3kstr.len() {
        // First byte is the content length
        let content_length = sc3kstr.bytes[i] as usize;
        if content_length == 0 {
            break;
        }

        // Detokenise the contents
        detokenise_line(&mut output, &sc3kstr.bytes[i..i + 5 + content_length], sc3kstr.cset);
        i += 5 + content_length;

        output.push('\x0a');
        i += 1;
    }

    output.shrink_to_fit();
    output
}

/// Tokenise a line of Unicode text into bytes for use in an SC-3000 BASIC file
pub fn tokenise_line(line: &str, cset: CharacterSet) -> Option<SC3000String> {
    let mut bytes = Vec::<u8>::new();
    bytes.push(0x00);	// placeholder - replace with line length later

    let space_i = line.find(' ')?;
    let lineno = line[0..space_i].parse::<u16>().ok()?;
    bytes.push((lineno & 0xff) as u8);
    bytes.push((lineno >> 8) as u8);

    // two unknown bytes
    bytes.push(0x00);
    bytes.push(0x00);

    let mut temp_line = String::new();

    let mut state = TokenState::Command;
    let mut j = space_i + 1;
    while j < line.len() {
        let c = line[j..].chars().next()?;

        match state {
            TokenState::Command => {
                if c.is_ascii() {
                    temp_line.push(c);
                    if c == '"' {
                        state = TokenState::QuotedString;
                    }
                    j += 1;
                    continue;
                }
                if let Some(command_num) = COMMANDLIST.iter().position(|t| line[j..].starts_with((*t).1)) {
                    if !temp_line.is_empty() {
                        bytes.extend(SC3000String::from_string(&temp_line, cset).bytes);
                        temp_line.clear();
                    }
                    bytes.push(COMMANDLIST[command_num].0);

                    // REM
                    if COMMANDLIST[command_num].0 == 0x90 {
                        state = TokenState::Remark;
                    } else {
                        state = TokenState::ASCIIAndFuncs;
                    }

                    j += COMMANDLIST[command_num].1.len();
                    continue;
                }
            },

            TokenState::Remark => {
                temp_line.push(c);

                j += 1;
                while !line.is_char_boundary(j) {
                    j += 1;
                }
                continue;
            },

            TokenState::ASCIIAndFuncs => {
                if c.is_ascii() {
                    temp_line.push(c);
                    if c == ':' {
                        state = TokenState::Command;
                    } else if c == '"' {
                        state = TokenState::QuotedString;
                    }
                    j += 1;
                    continue;
                }

                if let Some(func_num) = FUNCLIST.iter().position(|f| line[j..].starts_with((*f).1)) {
                    if !temp_line.is_empty() {
                        bytes.extend(SC3000String::from_string(&temp_line, cset).bytes);
                        temp_line.clear();
                    }
                    bytes.push(FUNCLIST[func_num].0);

                    j += FUNCLIST[func_num].1.len();
                    continue;
                }

                // Fallback to command codes
                if let Some(command_num) = COMMANDLIST.iter().position(|t| line[j..].starts_with((*t).1)) {
                    if !temp_line.is_empty() {
                        bytes.extend(SC3000String::from_string(&temp_line, cset).bytes);
                        temp_line.clear();
                    }
                    bytes.push(COMMANDLIST[command_num].0);

                    j += COMMANDLIST[command_num].1.len();
                    continue;
                }
            },

            TokenState::QuotedString => {
                temp_line.push(c);

                if c == '"' {
                    state = TokenState::ASCIIAndFuncs;
                }

                j += 1;
                while !line.is_char_boundary(j) {
                    j += 1;
                }
                continue;
            },

        }

        // Just continue on
        j += 1;
    }

    if !temp_line.is_empty() {
        bytes.extend(SC3000String::from_string(&temp_line, cset).bytes);
    }

    // replace the line length at the start of the line
    bytes[0] = (bytes.len() - 5) as u8;

    // Newline
    bytes.push(0x0d);

    Some(SC3000String {
        cset,
        bytes: bytes.into(),
    })
}

/// Tokenise a Unicode string into bytes for use in an SC-3000 BASIC file
pub fn tokenise(source: &str, cset: CharacterSet) -> Option<SC3000String> {
    let mut output = Vec::<u8>::with_capacity(source.len() / 10);

    for line in source.lines() {
        let s = tokenise_line(line, cset).unwrap();
        output.extend(&s.bytes);
    }

    output.shrink_to_fit();
    Some(SC3000String::from_cset(output.as_ref(), cset))
}
