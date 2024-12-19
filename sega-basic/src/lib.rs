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
//! TODO: a function to tokenise source code

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use sc3000_charset::{CharacterSet, SC3000String};

lazy_static! {
    static ref TOKENS: HashMap<u8, &'static str> = HashMap::from([
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
    ]);

    static ref FUNCS: HashMap<u8, &'static str> = HashMap::from([
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
    ]);
}

fn detokenise_line(output: &mut String, input: &[u8], cset: CharacterSet) {
    let mut use_funcs = false;
    let mut is_string = false;
    let mut temp_bytes = vec![];

    for b in input {
        // Ordinary ASCII characters
        if *b < 127 {
            temp_bytes.push(*b);
            if *b == b':' {
                use_funcs = false;
            } else if *b == b'"' {
                is_string = !is_string;
            }
            continue;
        }

        // Only process high-value bytes as characters if we're in a quoted string
        if is_string {
            temp_bytes.push(*b);
            continue;
        }

        if use_funcs {
            if let Some((_, funcname)) = FUNCS.get_key_value(b) {
                if !temp_bytes.is_empty() {
                    output.push_str(&SC3000String::from_cset(temp_bytes.as_slice(), cset).to_string());
                    temp_bytes.clear();
                }
                output.push_str(funcname);
                continue;
            }
        }
        use_funcs = true;

        if let Some((_, tokname)) = TOKENS.get_key_value(b) {
            if !temp_bytes.is_empty() {
                output.push_str(&SC3000String::from_cset(temp_bytes.as_slice(), cset).to_string());
                temp_bytes.clear();
            }
            output.push_str(tokname);

            if *b == 0x90 {
                is_string = true;
            }

            continue;
        }

        // ?
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
        // First byte is the line length (after the line number)
        let line_length = sc3kstr.bytes[i] as usize;
        if line_length == 0 {
            break;
        }
        i += 1;

        // Next two bytes are the line number
        let lineno = (sc3kstr.bytes[i] as u16) | ((sc3kstr.bytes[i + 1] as u16) << 8);
        i += 2;

        // Next two bytes?
        i += 2;

        // Print the line number
        output.push_str(&lineno.to_string());
        output.push(' ');

        // Detokenise the contents
        detokenise_line(&mut output, &sc3kstr.bytes[i..i + line_length], sc3kstr.cset);
        i += line_length;

        output.push('\x0a');
        i += 1;
    }

    output.shrink_to_fit();
    output
}
