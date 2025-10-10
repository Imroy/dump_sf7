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
//!   from regular ASCII text
//! - A 'function' byte code appears after a 0x80 byte
//! - Quoted text strings and anything after a REM or DATA statement can use the whole 8-bit [character set](sc3000_charset::CharacterSet)
//!
//! See [SegaBasicVersion] for tables of the statment and function values.

#[macro_use]
extern crate lazy_static;

use std::collections::{BTreeSet, HashMap};

use sc3000_charset::{CharacterSet, SC3000String};

/// The different versions of Sega BASIC
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum SegaBasicVersion {
    /// Sega SC-3000 BASIC Level 2 or 3 v1.0 (1983; cartridge)
    ///
    /// ## Statements
    /// |    | x0 | x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8 | x9 | xA | xB | xC | xD | xE | xF |
    /// |----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
    /// | **8x** | | | LIST | LLIST | AUTO | DELETE | RUN | CONT | LOAD | SAVE| VERIFY | NEW | RENUM | | | BOOT |
    /// | **9x** | REM | PRINT or ? | LPRINT or L? | DATA | DEF | INPUT | READ | STOP | END | LET | DIM | FOR | NEXT | GOTO | GOSUB | GO |
    /// | **Ax** | ON | RETURN | ERASE | CURSOR | IF | RESTORE | SCREEN | COLOR | LINE | SOUND | BEEP | CONSOLE | CLS | OUT | CALL | POKE |
    /// | **Bx** | PSET | PRESET | PAINT | BLINE | POSITION | HCOPY | SPRITE | PATTERN | CIRCLE | BCIRCLE | MAG | VPOKE | MOTOR | | | |
    /// | **Cx** | ^ | * | / | MOD | + | - | <> or >< | >= or => | <= or =< | > | < | = | NOT | AND | OR | XOR |
    /// | **Dx** | | | | | | | | | | | | | |
    /// | **Ex** | FN | TO | STEP | THEN | TAB | SPC | APPEND | OUTPUT |
    /// | **Fx** | | | | | | | | | | |
    ///
    /// ## Functions
    /// |    | x0 | x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8 | x9 | xA | xB | xC | xD | xE | xF |
    /// |----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
    /// | **8x** | ABS | RND | SIN | COS | TAN | ASN | ACS | ATN | LOG | LGT | LTW | EXP | RAD | DEG | PI | SQR |
    /// | **9x** | INT | SGN | ASC | LEN | VAL | PEEK | INP | FRE | VPEEK | STICK | STRIG | | | | | |
    /// | **Ax** | CHR$ | HEX$ | INKEY$ | LEFT$ | RIGHT$ | MID$ | STR$ | TIME$ |
    CartridgeBasic,

    /// Sega SC-3000 Disk BASIC v1.0p or v1.1p (1984; floppy)
    ///
    /// New or altered statements and functions are highlighted in bold.
    ///
    /// ## Statements
    /// |    | x0 | x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8 | x9 | xA | xB | xC | xD | xE | xF |
    /// |----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
    /// | **8x** | | INPUT | LIST | LLIST | AUTO | DELETE | RUN | CONT | **CLOAD** | **CSAVE** | VERIFY | NEW | RENUM | FILES | LFILES | BOOT |
    /// | **9x** | REM | PRINT or ? | LPRINT or L? | DATA | DEF | INPUT | READ | STOP | END | LET | DIM | FOR | NEXT | GOTO | GOSUB | GO |
    /// | **Ax** | ON | RETURN | ERASE | CURSOR | IF | RESTORE | SCREEN | COLOR | LINE | SOUND | BEEP | CONSOLE | CLS | OUT | CALL | POKE |
    /// | **Bx** | PSET | PRESET | PAINT | BLINE | POSITION | HCOPY | SPRITE | PATTERN | CIRCLE | BCIRCLE | MAG | VPOKE | MOTOR | OPEN | CLOSE | COMSET |
    /// | **Cx** | ^ | * | / | MOD | + | - | <> or >< | >= or => | <= or =< | > | < | = | NOT | AND | OR | XOR |
    /// | **Dx** | **CLOADM** | **CSAVEM** | **VERIFYM** | **SAVEM** | **LOADM** | **LIMIT** | **GET** | **PUT** | **DSKI$** | **DSKO$** | **KILL** | **SET** | **NAME** |
    /// | **Ex** | FN | TO | STEP | THEN | TAB | SPC | APPEND | OUTPUT |
    /// | **Fx** | **SAVE** | **LOAD** | | | | **MERGE** | **COMSAVE** | **COMLOAD** | **UTILITY** | **MAXFILE** |
    ///
    /// ## Functions
    /// |    | x0 | x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8 | x9 | xA | xB | xC | xD | xE | xF |
    /// |----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
    /// | **8x** | ABS | RND | SIN | COS | TAN | ASN | ACS | ATN | LOG | LGT | LTW | EXP | RAD | DEG | PI | SQR |
    /// | **9x** | INT | SGN | ASC | LEN | VAL | PEEK | INP | FRE | VPEEK | STICK | STRIG | **EOF** | **LOC** | **LOF** | **DSKF** | |
    /// | **Ax** | CHR$ | HEX$ | INKEY$ | LEFT$ | RIGHT$ | MID$ | STR$ | TIME$ |
    DiskBasic,
}

/// A whole Sega BASIC program, containing multiple lines
#[derive(Clone, Debug)]
pub struct SegaBasicProgram {
    /// Lines of BASIC source code
    lines: Vec<SegaBasicLine>,

    /// Version of BASIC
    bver: SegaBasicVersion,

    /// Character set
    cset: CharacterSet,
}

impl SegaBasicProgram {
    pub fn new(bver: SegaBasicVersion, cset: CharacterSet) -> Self {
        Self {
            lines: Vec::new(),
            bver,
            cset,
        }
    }

    /// Constructor from a slice of raw bytes
    pub fn from_bytes(bytes: &[u8], bver: SegaBasicVersion, cset: CharacterSet) -> Self {
        let mut lines = Vec::new();

        let mut i = 0;
        while i < bytes.len() {
            // First byte is the content length
            let content_length = bytes[i];
            if content_length == 0 {
                break;
            }

            lines.push(SegaBasicLine::new(
                &bytes[i..i + 5 + content_length as usize],
            ));
            i += 5 + content_length as usize;
            i += 1; // newline
        }

        Self { lines, bver, cset }
    }

    /// Tokenise a Unicode string into a new program
    pub fn tokenise(source: &str, bver: SegaBasicVersion, cset: CharacterSet) -> Self {
        let mut lines = Vec::new();
        for line_str in source.lines() {
            let line = SegaBasicLine::tokenise(line_str, bver, cset);
            lines.push(line);
        }

        Self { bver, cset, lines }
    }

    /// Return the tokenised bytes of the complete program
    pub fn bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();

        for line in &self.lines {
            output.extend(line.complete_bytes());
        }

        output
    }

    /// Detokenise the program into a Unicode string
    pub fn detokenise(&self) -> String {
        let mut output = String::new();

        for line in &self.lines {
            output.push_str(&line.detokenise(self.bver, self.cset));
            output.push('\n');
        }
        output.shrink_to_fit();

        output
    }

    /// Add a line to the program
    pub fn add_line(&mut self, line: &SegaBasicLine) {
        if let Some(index) = self.lines.iter().position(|l| l.lineno >= line.lineno) {
            self.lines.insert(index, line.clone());
        } else {
            self.lines.push(line.clone());
        }
    }

    /// Remove a line from the program
    ///
    /// Returns the line if it was found
    pub fn remove_line(&mut self, lineno: u16) -> Option<SegaBasicLine> {
        self.lines
            .iter()
            .position(|l| l.lineno == lineno)
            .map(|index| self.lines.remove(index))
    }

    /// Sort the list of lines of the program by their line number
    pub fn sort_lines(&mut self) {
        self.lines.sort_unstable_by(|a, b| a.lineno.cmp(&b.lineno));
    }

    /// Renumber the lines of the program
    ///
    /// Sorts the lines first.
    pub fn renumber(&mut self, start: u16, incr: u16) {
        self.lines.sort_unstable_by(|a, b| a.lineno.cmp(&b.lineno));

        let mut n = start;
        for line in &mut self.lines {
            line.lineno = n;

            n += incr;
        }
    }
}

/// A line of Sega BASIC code
#[derive(Clone, Debug)]
pub struct SegaBasicLine {
    /// Line number
    lineno: u16,

    /// Bytes of the line, after the line number
    content_bytes: Vec<u8>,
}

impl SegaBasicLine {
    /// Simple constructor from a slice of bytes
    pub fn new(line_bytes: &[u8]) -> Self {
        let lineno = (line_bytes[1] as u16) | ((line_bytes[2] as u16) << 8);
        Self {
            lineno,
            content_bytes: line_bytes[5..].to_vec(),
        }
    }

    /// Tokenise a line of Unicode text into bytes
    pub fn tokenise(line: &str, bver: SegaBasicVersion, cset: CharacterSet) -> Self {
        let mut j = 0;

        // Read the line number
        let mut lineno: u16 = 0;
        while j < line.len() && line[j..].chars().next().unwrap().is_ascii_digit() {
            lineno = (lineno * 10) + (line[j..].chars().next().unwrap() as u16 - '0' as u16);
            j += 1;
        }

        // Optional spaces
        while j < line.len() && line[j..].starts_with(' ') {
            j += 1;
        }

        let mut content_bytes = Vec::<u8>::with_capacity(line.len() / 5);
        let mut temp_line = String::new();

        let mut state = TokenState::default();
        'character: while j < line.len() {
            let c = line[j..].chars().next().unwrap();

            match state {
                TokenState::Statement => {
                    for length in ALIAS_LENGTHS.iter().rev() {
                        if j + length < line.len()
                            && let Some(code) = ALIASES.get(&line[j..j + length])
                        {
                            flush_line(&mut content_bytes, &mut temp_line, cset);
                            content_bytes.push(*code);

                            j += length;
                            continue 'character;
                        }
                    }

                    for length in STATEMENT_LENGTHS[bver as usize].iter().rev() {
                        if j + length < line.len()
                            && let Some(stmt_code) =
                                STATEMENT_CODES[bver as usize].get(&line[j..j + length])
                        {
                            flush_line(&mut content_bytes, &mut temp_line, cset);
                            content_bytes.push(*stmt_code);

                            // REM or DATA
                            if *stmt_code == 0x90 || *stmt_code == 0x93 {
                                state = TokenState::RemarkOrData;
                            }

                            j += length;
                            continue 'character;
                        }
                    }

                    for length in FUNC_LENGTHS[bver as usize].iter().rev() {
                        if j + length < line.len()
                            && let Some(func_code) =
                                FUNC_CODES[bver as usize].get(&line[j..j + length])
                        {
                            flush_line(&mut content_bytes, &mut temp_line, cset);
                            content_bytes.push(0x80);
                            content_bytes.push(*func_code);

                            j += length;
                            continue 'character;
                        }
                    }

                    if c.is_ascii() {
                        temp_line.push(c);
                        if c == '"' {
                            state = TokenState::QuotedString;
                        }
                        j += 1;
                        continue 'character;
                    }
                }

                // we never use this state when tokenising
                TokenState::Function => (),

                TokenState::RemarkOrData => {
                    temp_line.push(c);

                    j += 1;
                    while !line.is_char_boundary(j) {
                        j += 1;
                    }
                    continue 'character;
                }

                TokenState::QuotedString => {
                    temp_line.push(c);

                    if c == '"' {
                        state = TokenState::Statement;
                    }

                    j += 1;
                    while !line.is_char_boundary(j) {
                        j += 1;
                    }
                    continue 'character;
                }
            }

            // Just continue on
            j += 1;
        }

        flush_line(&mut content_bytes, &mut temp_line, cset);
        content_bytes.shrink_to_fit();

        Self {
            lineno,
            content_bytes,
        }
    }

    /// Return the complete line as a vector of bytes
    pub fn complete_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::with_capacity(self.content_bytes.len() + 6);

        // Content length
        bytes.push(self.content_bytes.len() as u8);

        // Line number
        bytes.push((self.lineno & 0xff) as u8);
        bytes.push((self.lineno >> 8) as u8);

        // two unknown bytes
        bytes.push(0x00);
        bytes.push(0x00);

        // Contents
        bytes.extend(self.content_bytes.clone());

        // Newline
        bytes.push(0x0d);

        bytes
    }

    /// Detokenise this line of BASIC source code into a Unicode string
    pub fn detokenise(&self, bver: SegaBasicVersion, cset: CharacterSet) -> String {
        let mut temp_bytes = Vec::new();
        let mut output = String::with_capacity(self.content_bytes.len() * 5);

        // Print the line number
        output.push_str(&self.lineno.to_string());
        output.push(' ');

        let mut state = TokenState::default();
        let mut j = 0;
        while j < self.content_bytes.len() {
            let b = self.content_bytes[j];
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
                        flush_bytes(&mut output, &mut temp_bytes, cset);
                        output.push_str(stmtname);

                        // REM
                        if b == 0x90 || b == 0x93 {
                            state = TokenState::RemarkOrData;
                        }

                        j += 1;
                        continue;
                    }
                }

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
                        flush_bytes(&mut output, &mut temp_bytes, cset);
                        output.push_str(funcname);

                        state = TokenState::Statement;
                        j += 1;
                        continue;
                    }
                }

                TokenState::RemarkOrData => {
                    // Any 8-bit character
                    temp_bytes.push(b);

                    j += 1;
                    continue;
                }

                TokenState::QuotedString => {
                    // Any 8-bit character
                    temp_bytes.push(b);

                    // End the quoted string
                    if b == b'"' {
                        state = TokenState::Statement;
                    }

                    j += 1;
                    continue;
                }
            }

            // ?
            j += 1;
        }

        flush_bytes(&mut output, &mut temp_bytes, cset);
        output.shrink_to_fit();

        output
    }
}

/// States of the tokeniser/detokeniser
#[derive(Copy, Clone, Debug, Default)]
enum TokenState {
    #[default]
    Statement,
    Function,
    RemarkOrData,
    QuotedString,
}

static ALIAS_LIST: [(&str, u8); 5] = [
    ("?", 0x91), ("L?", 0x92),
    ("><", 0xc6), ("=>", 0xc7),
    ("=<", 0xc8),
];

static STATEMENTS_CARTRIDGE: [(u8, &str); 78] = [
    (0x82, "LIST"), (0x83, "LLIST"),
    (0x84, "AUTO"), (0x85, "DELETE"), (0x86, "RUN"), (0x87, "CONT"),
    (0x88, "LOAD"), (0x89, "SAVE"), (0x8a, "VERIFY"), (0x8b, "NEW"),
    (0x8c, "RENUM"),
    (0x90, "REM"), (0x91, "PRINT"), (0x92, "LPRINT"), (0x93, "DATA"),
    (0x94, "DEF"), (0x95, "INPUT"), (0x96, "READ"), (0x97, "STOP"),
    (0x98, "END"), (0x99, "LET"), (0x9a, "DIM"), (0x9b, "FOR"),
    (0x9c, "NEXT"), (0x9d, "GOTO"), (0x9e, "GOSUB"), (0x9f, "GO"),
    (0xa0, "ON"), (0xa1, "RETURN"), (0xa2, "ERASE"), (0xa3, "CURSOR"),
    (0xa4, "IF"), (0xa5, "RESTORE"), (0xa6, "SCREEN"), (0xa7, "COLOR"),
    (0xa8, "LINE"), (0xa9, "SOUND"), (0xaa, "BEEP"), (0xab, "CONSOLE"),
    (0xac, "CLS"), (0xad, "OUT"), (0xae, "CALL"), (0xaf, "POKE"),
    (0xb0, "PSET"), (0xb1, "PRESET"), (0xb2, "PAINT"), (0xb3, "BLINE"),
    (0xb4, "POSITION"), (0xb5, "HCOPY"), (0xb6, "SPRITE"), (0xb7, "PATTERN"),
    (0xb8, "CIRCLE"), (0xb9, "BCIRCLE"), (0xba, "MAG"), (0xbb, "VPOKE"),
    (0xbc, "MOTOR"),
    (0xc0, "^"), (0xc1, "*"), (0xc2, "/"), (0xc3, "MOD"),
    (0xc4, "+"), (0xc5, "-"), (0xc6, "<>"), (0xc7, ">="),
    (0xc8, "<="), (0xc9, ">"), (0xca, "<"), (0xcb, "="),
    (0xcc, "NOT"), (0xcd, "AND"), (0xce, "OR"), (0xcf, "XOR"),
    (0xe0, "FN"), (0xe1, "TO"), (0xe2, "STEP"), (0xe3, "THEN"),
    (0xe4, "TAB"), (0xe5, "SPC"),
];

static STATEMENTS_DISK: [(u8, &str); 107] = [
    (0x81, "INPUT$"), (0x82, "LIST"), (0x83, "LLIST"),
    (0x84, "AUTO"), (0x85, "DELETE"), (0x86, "RUN"), (0x87, "CONT"),
    (0x88, "CLOAD"), (0x89, "CSAVE"), (0x8a, "VERIFY"), (0x8b, "NEW"),
    (0x8c, "RENUM"), (0x8d, "FILES"), (0x8e, "LFILES"), (0x8f, "BOOT"),
    (0x90, "REM"), (0x91, "PRINT"), (0x92, "LPRINT"), (0x93, "DATA"),
    (0x94, "DEF"), (0x95, "INPUT"), (0x96, "READ"), (0x97, "STOP"),
    (0x98, "END"), (0x99, "LET"), (0x9a, "DIM"), (0x9b, "FOR"),
    (0x9c, "NEXT"), (0x9d, "GOTO"), (0x9e, "GOSUB"), (0x9f, "GO"),
    (0xa0, "ON"), (0xa1, "RETURN"), (0xa2, "ERASE"), (0xa3, "CURSOR"),
    (0xa4, "IF"), (0xa5, "RESTORE"), (0xa6, "SCREEN"), (0xa7, "COLOR"),
    (0xa8, "LINE"), (0xa9, "SOUND"), (0xaa, "BEEP"), (0xab, "CONSOLE"),
    (0xac, "CLS"), (0xad, "OUT"), (0xae, "CALL"), (0xaf, "POKE"),
    (0xb0, "PSET"), (0xb1, "PRESET"), (0xb2, "PAINT"), (0xb3, "BLINE"),
    (0xb4, "POSITION"), (0xb5, "HCOPY"), (0xb6, "SPRITE"), (0xb7, "PATTERN"),
    (0xb8, "CIRCLE"), (0xb9, "BCIRCLE"), (0xba, "MAG"), (0xbb, "VPOKE"),
    (0xbc, "MOTOR"), (0xbd, "OPEN"), (0xbe, "CLOSE"), (0xbf, "COMSET"),
    (0xc0, "^"), (0xc1, "*"), (0xc2, "/"), (0xc3, "MOD"),
    (0xc4, "+"), (0xc5, "-"), (0xc6, "<>"), (0xc7, ">="),
    (0xc8, "<="), (0xc9, ">"), (0xca, "<"), (0xcb, "="),
    (0xcc, "NOT"), (0xcd, "AND"), (0xce, "OR"), (0xcf, "XOR"),
    (0xd0, "CLOADM"), (0xd1, "CSAVEM"), (0xd2, "VERIFYM"), (0xd3, "SAVEM"),
    (0xd4, "LOADM"), (0xd5, "LIMIT"), (0xd6, "GET"), (0xd7, "PUT"),
    (0xd8, "DSKI$"), (0xd9, "DSKO$"), (0xda, "KILL"), (0xdb, "SET"),
    (0xdc, "NAME"),
    (0xe0, "FN"), (0xe1, "TO"), (0xe2, "STEP"), (0xe3, "THEN"),
    (0xe4, "TAB"), (0xe5, "SPC"), (0xe6, "APPEND"), (0xe7, "OUTPUT"),
    (0xf0, "SAVE"), (0xf1, "LOAD"),
    (0xf5, "MERGE"), (0xf6, "COMSAVE"), (0xf7, "COMLOAD"),
    (0xf8, "UTILITY"), (0xf9, "MAXFILE"),
];

static FUNCS_CARTRIDGE: [(u8, &str); 35] = [
    (0x80, "ABS"), (0x81, "RND"), (0x82, "SIN"), (0x83, "COS"),
    (0x84, "TAN"), (0x85, "ASN"), (0x86, "ACS"), (0x87, "ATN"),
    (0x88, "LOG"), (0x89, "LGT"), (0x8a, "LTW"), (0x8b, "EXP"),
    (0x8c, "RAD"), (0x8d, "DEG"), (0x8e, "PI"), (0x8f, "SQR"),
    (0x90, "INT"), (0x91, "SGN"), (0x92, "ASC"), (0x93, "LEN"),
    (0x94, "VAL"), (0x95, "PEEK"), (0x96, "INP"), (0x97, "FRE"),
    (0x98, "VPEEK"), (0x99, "STICK"), (0x9a, "STRIG"),
    (0xa0, "CHR$"), (0xa1, "HEX$"), (0xa2, "INKEY$"), (0xa3, "LEFT$"),
    (0xa4, "RIGHT$"), (0xa5, "MID$"), (0xa6, "STR$"), (0xa7, "TIME$"),
];

static FUNCS_DISK: [(u8, &str); 39] = [
    (0x80, "ABS"), (0x81, "RND"), (0x82, "SIN"), (0x83, "COS"),
    (0x84, "TAN"), (0x85, "ASN"), (0x86, "ACS"), (0x87, "ATN"),
    (0x88, "LOG"), (0x89, "LGT"), (0x8a, "LTW"), (0x8b, "EXP"),
    (0x8c, "RAD"), (0x8d, "DEG"), (0x8e, "PI"), (0x8f, "SQR"),
    (0x90, "INT"), (0x91, "SGN"), (0x92, "ASC"), (0x93, "LEN"),
    (0x94, "VAL"), (0x95, "PEEK"), (0x96, "INP"), (0x97, "FRE"),
    (0x98, "VPEEK"), (0x99, "STICK"), (0x9a, "STRIG"), (0x9b, "EOF"),
    (0x9c, "LOC"), (0x9d, "LOF"), (0x9e, "DSKF"),
    (0xa0, "CHR$"), (0xa1, "HEX$"), (0xa2, "INKEY$"), (0xa3, "LEFT$"),
    (0xa4, "RIGHT$"), (0xa5, "MID$"), (0xa6, "STR$"), (0xa7, "TIME$"),
];

lazy_static! {
    static ref ALIASES: HashMap<&'static str, u8> = HashMap::from(ALIAS_LIST);
    static ref ALIAS_LENGTHS: Vec<usize> = ALIAS_LIST
        .iter()
        .map(|(k, _)| k.len())
        .collect::<BTreeSet<usize>>()
        .iter()
        .copied()
        .collect::<Vec<usize>>();

    static ref STATEMENTS: [ HashMap<u8, &'static str>; 2 ] = [
        // BASIC Level 2 or 3
        HashMap::from(STATEMENTS_CARTRIDGE),

        // Disk BASIC v1.0p or v1.1p
        HashMap::from(STATEMENTS_DISK),
    ];
    static ref STATEMENT_LENGTHS: [ Vec<usize>; 2 ] = [
        STATEMENTS_CARTRIDGE
            .iter()
            .map(|(_, v)| v.len())
            .collect::<BTreeSet<usize>>()
            .iter()
            .copied()
            .collect(),
        STATEMENTS_DISK
            .iter()
            .map(|(_, v)| v.len())
            .collect::<BTreeSet<usize>>()
            .iter()
            .copied()
            .collect(),
    ];
    static ref STATEMENT_CODES: [ HashMap<&'static str, u8>; 2 ] = [
        STATEMENTS_CARTRIDGE
            .iter()
            .map(|(k, v)| (*v, *k))
            .collect(),
        STATEMENTS_DISK
            .iter()
            .map(|(k, v)| (*v, *k))
            .collect(),
    ];

    static ref FUNCS: [ HashMap<u8, &'static str>; 2 ] = [
        // BASIC Level 2 or 3
        HashMap::from(FUNCS_CARTRIDGE),

        // Disk BASIC v1.0p or v1.1p
        HashMap::from(FUNCS_DISK),
    ];
    static ref FUNC_LENGTHS: [ Vec<usize>; 2 ] = [
        FUNCS_CARTRIDGE
            .iter()
            .map(|(_, v)| v.len())
            .collect::<BTreeSet<usize>>()
            .iter()
            .copied()
            .collect(),
        FUNCS_DISK
            .iter()
            .map(|(_, v)| v.len())
            .collect::<BTreeSet<usize>>()
            .iter()
            .copied()
            .collect(),
    ];
    static ref FUNC_CODES: [ HashMap<&'static str, u8>; 2 ] = [
        FUNCS_CARTRIDGE
            .iter()
            .map(|(k, v)| (*v, *k))
            .collect(),
        FUNCS_DISK
            .iter()
            .map(|(k, v)| (*v, *k))
            .collect(),
    ];
}

fn flush_bytes(output: &mut String, temp_bytes: &mut Vec<u8>, cset: CharacterSet) {
    if temp_bytes.is_empty() {
        return;
    }

    output.push_str(&SC3000String::from_bytes(temp_bytes.as_slice(), cset).to_string());
    temp_bytes.clear();
}

fn flush_line(bytes: &mut Vec<u8>, temp_line: &mut String, cset: CharacterSet) {
    if temp_line.is_empty() {
        return;
    }

    //eprintln!("Adding line \"{}\" to bytes", temp_line);
    bytes.extend(SC3000String::from_string(temp_line, cset).bytes);
    temp_line.clear();
}
