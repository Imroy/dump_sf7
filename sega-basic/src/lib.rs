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
//! - Content is mostly 'statement' byte codes with ASCII characters
//! - Statement and function codes have their high bit set, so they can be easily distinguished
//! from regular ASCII text.
//! - A 'function' byte code appears after a 0x80 byte
//! - After a colon (':') another statement is given and the format restarts.
//! - Quoted text strings and anything after a REMark or DATA statement can use the whole 8-bit [character set](sc3000_charset).
//!
//! ## Statements
//!
//! Disk BASIC:
//! - Added INPUT (0x81)
//! - Renamed SAVE/LOAD to CSAVE/CLOAD (0x88, 0x89)
//! - Added FILES, LFILES, BOOT (0x8d-8f)
//! - Added OPEN, CLOSE, COMSET (0xbd-bf)
//! - Added the 0xd0-0xdc and 0xf0-0xf9 lines
//! - Added APPEND and OUTPUT (0xe6, 0xe7)
//!
//! |    | x0 | x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8 | x9 | xA | xB | xC | xD | xE | xF |
//! |----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
//! | **8x** | | INPUT | LIST | LLIST | AUTO | DELETE | RUN | CONT | CLOAD (was LOAD) | CSAVE (was SAVE)| VERIFY | NEW | RENUM | FILES | LFILES | BOOT |
//! | **9x** | REM | PRINT or ? | LPRINT or L? | DATA | DEF | INPUT | READ | STOP | END | LET | DIM | FOR | NEXT | GOTO | GOSUB | GO |
//! | **Ax** | ON | RETURN | ERASE | CURSOR | IF | RESTORE | SCREEN | COLOR | LINE | SOUND | BEEP | CONSOLE | CLS | OUT | CALL | POKE |
//! | **Bx** | PSET | PRESET | PAINT | BLINE | POSITION | HCOPY | SPRITE | PATTERN | CIRCLE | BCIRCLE | MAG | VPOKE | MOTOR | OPEN | CLOSE | COMSET |
//! | **Cx** | ^ | * | / | MOD | + | - | <> or >< | >= or => | <= or =< | > | < | = | NOT | AND | OR | XOR |
//! | **Dx** | CLOADM | CSAVEM | VERIFYM | SAVEM | LOADM | LIMIT | GET | PUT | DSKI$ | DSKO$ | KILL | SET | NAME |
//! | **Ex** | FN | TO | STEP | THEN | TAB | SPC | APPEND | OUTPUT |
//! | **Fx** | SAVE | LOAD | | | | MERGE | COMSAVE | COMLOAD | UTILITY | MAXFILE |
//!
//! ## Functions
//!
//! Disk BASIC added EOF, LOC, LOF, and DSKF (0x9b-0x9e).
//!
//! |    | x0 | x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8 | x9 | xA | xB | xC | xD | xE | xF |
//! |----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
//! | **8x** | ABS | RND | SIN | COS | TAN | ASN | ACS | ATN | LOG | LGT | LTW | EXP | RAD | DEG | PI | SQR |
//! | **9x** | INT | SGN | ASC | LEN | VAL | PEEK | INP | FRE | VPEEK | STICK | STRIG | EOF | LOC | LOF | DSKF | |
//! | **Ax** | CHR$ | HEX$ | INKEY$ | LEFT$ | RIGHT$ | MID$ | STR$ | TIME$ |


#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use sc3000_charset::{CharacterSet, SC3000String};

/// The different versions of Sega BASIC
#[repr(u8)]
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub enum SegaBasicVersion {
    /// Sega SC-3000 BASIC Level 2 or 3 v1.0 (1983; cartridge)
    CartridgeBasic,

    /// Sega SC-3000 Disk BASIC v1.0p or v1.1p (1984; floppy)
    DiskBasic,
}


lazy_static! {
    static ref STATEMENTS: [ HashMap<u8, &'static str>; 2 ] = [
        // BASIC Level 2 or 3
        HashMap::from([
            ( 0x82, "LIST" ),  ( 0x83, "LLIST" ),
            ( 0x84, "AUTO" ), ( 0x85, "DELETE" ), ( 0x86, "RUN" ), ( 0x87, "CONT" ),
            ( 0x88, "LOAD" ), ( 0x89, "SAVE" ), ( 0x8a, "VERIFY" ), ( 0x8b, "NEW" ),
            ( 0x8c, "RENUM" ),

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
            ( 0xbc, "MOTOR" ),

            ( 0xc0, "^" ), ( 0xc1, "*" ), ( 0xc2, "/" ), ( 0xc3, "MOD" ),
            ( 0xc4, "+" ), ( 0xc5, "-" ), ( 0xc6, "<>" ), ( 0xc7, ">=" ),
            ( 0xc8, "<=" ), ( 0xc9, ">" ), ( 0xca, "<" ), ( 0xcb, "=" ),
            ( 0xcc, "NOT" ), ( 0xcd, "AND" ), ( 0xce, "OR" ), ( 0xcf, "XOR" ),

            ( 0xe0, "FN" ), ( 0xe1, "TO" ), ( 0xe2, "STEP" ), ( 0xe3, "THEN" ),
            ( 0xe4, "TAB" ), ( 0xe5, "SPC" ),
        ]),

        // Disk BASIC v1.0p or v1.1p
        HashMap::from([
            ( 0x81, "INPUT$" ), ( 0x82, "LIST" ), ( 0x83, "LLIST" ),
            ( 0x84, "AUTO" ), ( 0x85, "DELETE" ), ( 0x86, "RUN" ), ( 0x87, "CONT" ),
            ( 0x88, "CLOAD" ), ( 0x89, "CSAVE" ), ( 0x8a, "VERIFY" ), ( 0x8b, "NEW" ),
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

            ( 0xd0, "CLOADM" ), ( 0xd1, "CSAVEM" ), ( 0xd2, "VERIFYM" ), ( 0xd3, "SAVEM" ),
            ( 0xd4, "LOADM" ), ( 0xd5, "LIMIT" ), ( 0xd6, "GET" ), ( 0xd7, "PUT" ),
            ( 0xd8, "DSKI$" ), ( 0xd9, "DSKO$" ), ( 0xda, "KILL" ), ( 0xdb, "SET" ),
            ( 0xdc, "NAME" ),

            ( 0xe0, "FN" ), ( 0xe1, "TO" ), ( 0xe2, "STEP" ), ( 0xe3, "THEN" ),
            ( 0xe4, "TAB" ), ( 0xe5, "SPC" ), ( 0xe6, "APPEND" ), ( 0xe7, "OUTPUT" ),

            ( 0xf0, "SAVE" ), ( 0xf1, "LOAD" ),
            ( 0xf5, "MERGE" ), ( 0xf6, "COMSAVE" ), ( 0xf7, "COMLOAD" ),
            ( 0xf8, "UTILITY" ), ( 0xf9, "MAXFILE" ),
        ]),
    ];

    static ref ALIASES: [ ( &'static str, u8); 5 ] = [
        ( "?", 0x91 ), ( "L?", 0x92 ),
        ( "><", 0xc6 ), ( "=>", 0xc7 ), ( "=<", 0xc8 ),
    ];

    static ref FUNCS: [ HashMap<u8, &'static str>; 2 ] = [
        // BASIC Level 2 or 3
        HashMap::from([
            ( 0x80, "ABS" ), ( 0x81, "RND" ), ( 0x82, "SIN" ), ( 0x83, "COS" ),
            ( 0x84, "TAN" ), ( 0x85, "ASN" ), ( 0x86, "ACS" ), ( 0x87, "ATN" ),
            ( 0x88, "LOG" ), ( 0x89, "LGT" ), ( 0x8a, "LTW" ), ( 0x8b, "EXP" ),
            ( 0x8c, "RAD" ), ( 0x8d, "DEG" ), ( 0x8e, "PI" ), ( 0x8f, "SQR" ),

            ( 0x90, "INT" ), ( 0x91, "SGN" ), ( 0x92, "ASC" ), ( 0x93, "LEN" ),
            ( 0x94, "VAL" ), ( 0x95, "PEEK" ), ( 0x96, "INP" ), ( 0x97, "FRE" ),
            ( 0x98, "VPEEK" ), ( 0x99, "STICK" ), ( 0x9a, "STRIG" ),

            ( 0xa0, "CHR$" ), ( 0xa1, "HEX$" ), ( 0xa2, "INKEY$" ), ( 0xa3, "LEFT$" ),
            ( 0xa4, "RIGHT$" ), ( 0xa5, "MID$" ), ( 0xa6, "STR$" ), ( 0xa7, "TIME$" ),
        ]),

        // Disk BASIC v1.0p or v1.1p
        HashMap::from([
            ( 0x80, "ABS" ), ( 0x81, "RND" ), ( 0x82, "SIN" ), ( 0x83, "COS" ),
            ( 0x84, "TAN" ), ( 0x85, "ASN" ), ( 0x86, "ACS" ), ( 0x87, "ATN" ),
            ( 0x88, "LOG" ), ( 0x89, "LGT" ), ( 0x8a, "LTW" ), ( 0x8b, "EXP" ),
            ( 0x8c, "RAD" ), ( 0x8d, "DEG" ), ( 0x8e, "PI" ), ( 0x8f, "SQR" ),

            ( 0x90, "INT" ), ( 0x91, "SGN" ), ( 0x92, "ASC" ), ( 0x93, "LEN" ),
            ( 0x94, "VAL" ), ( 0x95, "PEEK" ), ( 0x96, "INP" ), ( 0x97, "FRE" ),
            ( 0x98, "VPEEK" ), ( 0x99, "STICK" ), ( 0x9a, "STRIG" ), ( 0x9b, "EOF" ),
            ( 0x9c, "LOC" ), ( 0x9d, "LOF" ), ( 0x9e, "DSKF" ),

            ( 0xa0, "CHR$" ), ( 0xa1, "HEX$" ), ( 0xa2, "INKEY$" ), ( 0xa3, "LEFT$" ),
            ( 0xa4, "RIGHT$" ), ( 0xa5, "MID$" ), ( 0xa6, "STR$" ), ( 0xa7, "TIME$" ),
        ]),
    ];
}


#[derive(Copy, Clone, Debug)]
enum TokenState {
    Statement,
    Function,
    RemarkOrData,
    QuotedString,
}

fn flush_bytes(output: &mut String, temp_bytes: &mut Vec::<u8>, cset: CharacterSet) {
    if temp_bytes.is_empty() {
        return;
    }

    output.push_str(&SC3000String::from_cset(temp_bytes.as_slice(), cset).to_string());
    temp_bytes.clear();
}

fn detokenise_line(output: &mut String, line: &[u8], bver: SegaBasicVersion, cset: CharacterSet) {
    let mut temp_bytes = vec![];

    // We don't use the content length in the first byte

    // Next two bytes are the line number
    let lineno = (line[1] as u16) | ((line[2] as u16) << 8);

    // Next two bytes?

    // Print the line number
    output.push_str(&lineno.to_string());
    output.push(' ');

    let mut state = TokenState::Statement;
    let mut j = 5;
    while j < line.len() {
        let b = line[j];
        match state {
            TokenState::Statement => {
                // Ordinary ASCII character
                if b < 128 {
                    temp_bytes.push(b);
                    if b == b'"' {
                        state = TokenState::QuotedString;
                    }

                    j += 1;
                    continue;
                }

                if b == 0x80 {
                    state = TokenState::Function;
                    j += 1;
                    continue;
                }

                if let Some(stmtname) = STATEMENTS[bver as usize].get(&b) {
                    flush_bytes(output, &mut temp_bytes, cset);
                    output.push_str(stmtname);

                    // REM
                    if b == 0x90 || b == 0x93 {
                        state = TokenState::RemarkOrData;
                    }

                    j += 1;
                    continue;
                }
            },

            TokenState::Function => {
                // Ordinary ASCII character
                if b < 128 {
                    temp_bytes.push(b);
                    if b == b':' {
                        state = TokenState::Statement;
                    } else if b == b'"' {
                        state = TokenState::QuotedString;
                    }

                    j += 1;
                    continue;
                }

                if let Some(funcname) = FUNCS[bver as usize].get(&b) {
                    flush_bytes(output, &mut temp_bytes, cset);
                    output.push_str(funcname);

                    state = TokenState::Statement;
                    j += 1;
                    continue;
                }
            },

            TokenState::RemarkOrData => {
                // Any 8-bit character
                temp_bytes.push(b);

                j += 1;
                continue;
            },

            TokenState::QuotedString => {
                // Any 8-bit character
                temp_bytes.push(b);

                // End the quoted string
                if b == b'"' {
                    state = TokenState::Statement;
                }

                j += 1;
                continue;
            },
        }

        // ?
        j += 1;
    }

    flush_bytes(output, &mut temp_bytes, cset);
}

/// Detokenise a byte slice of data holding BASIC source code into a Unicode string
pub fn detokenise(sc3kstr: &SC3000String, bver: SegaBasicVersion) -> String {
    let mut output = String::with_capacity(sc3kstr.len() * 10);

    let mut i = 0;
    while i < sc3kstr.len() {
        // First byte is the content length
        let content_length = sc3kstr.bytes[i] as usize;
        if content_length == 0 {
            break;
        }

        // Detokenise the contents
        detokenise_line(&mut output, &sc3kstr.bytes[i..i + 5 + content_length], bver, sc3kstr.cset);
        i += 5 + content_length;

        output.push('\x0a');
        i += 1;
    }

    output.shrink_to_fit();
    output
}

fn flush_line(bytes: &mut Vec::<u8>, temp_line: &mut String, cset: CharacterSet) {
    if temp_line.is_empty() {
        return;
    }

    //eprintln!("Adding line \"{}\" to bytes", temp_line);
    bytes.extend(SC3000String::from_string(&temp_line, cset).bytes);
    temp_line.clear();
}

/// Tokenise a line of Unicode text into bytes for use in an SC-3000 BASIC file
pub fn tokenise_line(line: &str, bver: SegaBasicVersion, cset: CharacterSet) -> Option<SC3000String> {
    let mut bytes = Vec::<u8>::with_capacity(line.len() / 10);
    bytes.push(0x00);	// placeholder - replace with line length later

    let space_i = line.find(' ')?;
    let lineno = line[0..space_i].parse::<u16>().ok()?;
    bytes.push((lineno & 0xff) as u8);
    bytes.push((lineno >> 8) as u8);

    // two unknown bytes
    bytes.push(0x00);
    bytes.push(0x00);

    let mut temp_line = String::new();

    let mut state = TokenState::Statement;
    let mut j = space_i + 1;
    while j < line.len() {
        let c = line[j..].chars().next()?;

        match state {
            TokenState::Statement => {
                if let Some(num) = ALIASES.iter().position(|alias| line[j..].starts_with((*alias).0)) {
                    flush_line(&mut bytes, &mut temp_line, cset);
                    bytes.push(ALIASES[num].1);

                    j += ALIASES[num].0.len();
                    continue;
                }

                if let Some((&stmt_code, &statement)) = (&STATEMENTS[bver as usize]).iter()
                    .filter(|&(_, v)| line[j..].starts_with(v))
                    .max_by_key(|&(_, v)| v.len()) {
                        flush_line(&mut bytes, &mut temp_line, cset);
                        bytes.push(stmt_code);

                        // REM
                        if stmt_code == 0x90 || stmt_code == 0x93 {
                            state = TokenState::RemarkOrData;
                        }

                        j += statement.len();
                        continue;
                    }

                if let Some((&func_code, &func_name)) = (&FUNCS[bver as usize]).iter()
                    .filter(|&(_, v)| line[j..].starts_with(v))
                    .max_by_key(|&(_, v)| v.len()) {
                        flush_line(&mut bytes, &mut temp_line, cset);
                        bytes.push(0x80);
                        bytes.push(func_code);

                        j += func_name.len();
                        continue;
                    }

                if c.is_ascii() {
                    temp_line.push(c);
                    if c == '"' {
                        state = TokenState::QuotedString;
                    }
                    j += 1;
                    continue;
                }
            },

            // we never use this state when tokenising
            TokenState::Function => (),

            TokenState::RemarkOrData => {
                temp_line.push(c);

                j += 1;
                while !line.is_char_boundary(j) {
                    j += 1;
                }
                continue;
            },

            TokenState::QuotedString => {
                temp_line.push(c);

                if c == '"' {
                    state = TokenState::Statement;
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

    flush_line(&mut bytes, &mut temp_line, cset);

    // replace the line length at the start of the line
    bytes[0] = (bytes.len() - 5) as u8;

    // Newline
    bytes.push(0x0d);
    bytes.shrink_to_fit();

    Some(SC3000String {
        cset,
        bytes: bytes.into(),
    })
}

/// Tokenise a Unicode string into bytes for use in an SC-3000 BASIC file
pub fn tokenise(source: &str, bver: SegaBasicVersion, cset: CharacterSet) -> Option<SC3000String> {
    let mut output = Vec::<u8>::with_capacity(source.len() / 10);

    for line in source.lines() {
        let s = tokenise_line(line, bver, cset).unwrap();
        output.extend(&s.bytes);
    }

    output.shrink_to_fit();
    Some(SC3000String::from_cset(output.as_ref(), cset))
}
