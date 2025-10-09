/*
  sc3000-charset
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

//! Sega SC-3000 character map conversion

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConversionError {
    NonRepresentableCharacterFound,
    BufferTooSmall,
}

type Result<T> = core::result::Result<T, ConversionError>;

/// The character sets that different models of the SC-3000 used
///
/// Using data from [Wikipedia](https://en.wikipedia.org/wiki/Sega_SC-3000_character_set) and
/// [SMS Power!](https://www.smspower.org/Development/SC-3000Font).
/// Characters listed as 'unused' or which cannot be represented will be replaced with
/// [**U+FFFD REPLACEMENT CHARACTER**](https://en.wikipedia.org/wiki/Specials_(Unicode_block)#Replacement_character).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CharacterSet {
    /// Character set used by Japanese models
    ///
    /// |    | x0 | x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8 | x9 | xA | xB | xC | xD | xE | xF |
    /// |----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
    /// | **2x** | **SP** | ! | " | # | $ | % | & | ' | ( | ) | * | + | , | - | . | / |
    /// | **3x** | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | : | ; | < | = | > | ? |
    /// | **4x** | @ | A | B | C | D | E | F | G | H | I | J | K | L | M | N | O |
    /// | **5x** | P | Q | R | S | T | U | V | W | X | Y | Z | [ | ¥ | ] | ^ | π |
    /// | **6x** | ` | a | b | c | d | e | f | g | h | i | j | k | l | m | n | o |
    /// | **7x** | p | q | r | s | t | u | v | w | x | y | z | { | \| | } | ~ | **DEL** |
    /// | **8x** | │ | ─ | ┴ | ┬ | ┤ | ├ | ┌ | └ | ┐ | ┘ | ╭ | ╰ | ╮ | ╯ | ↑ | ← |
    /// | **9x** | ▒ | ╳ | ┼ | ╱ | ╲ | ◢ | ◣ | ◥ | ◤ | ▁ | ▂ | ▄ | ▀ | 🮂 | ▔ | ▏ |
    /// | **Ax** |   | 。 | 「 | 」 | 、 | ・ | ヲ | ァ | ィ | ゥ | ェ | ォ | ャ | ュ | ョ | ッ |
    /// | **Bx** | ー | ア | イ | ウ | エ | オ | カ | キ | ク | ケ | コ | サ | シ | ス | セ | ソ |
    /// | **Cx** | タ | チ | ツ | テ | ト | ナ | ニ | ヌ | ネ | ノ | ハ | ヒ | フ | ヘ | ホ | マ |
    /// | **Dx** | ミ | ム | メ | モ | ヤ | ユ | ヨ | ラ | リ | ル | レ | ロ | ワ | ン | ゛ | ゜
    /// | **Ex** | ▎ | ▌ | ▐ | 🮇 | ▕ | █ | | | | | ▞ | ○ | ● | 年 | 月 | 日 |
    /// | **Fx** | 火 | 水 | 木 | 金 | 土 | ♠ | ♥ | ♦ | ♣ | | | | | 🯅 | ÷ |
    ///
    /// Note:
    /// - 0x5c is `¥` instead of `\`, and 0x5f is `π` instead of `_`, as in standard ASCII
    Japanese,

    /// Character set used by 'Export' models i.e for use with Western European languages
    ///
    /// |    | x0 | x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8 | x9 | xA | xB | xC | xD | xE | xF |
    /// |----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
    /// | **2x** | **SP** | ! | " | # | $ | % | & | ' | ( | ) | * | + | , | - | . | / |
    /// | **3x** | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | : | ; | < | = | > | ? |
    /// | **4x** | @ | A | B | C | D | E | F | G | H | I | J | K | L | M | N | O |
    /// | **5x** | P | Q | R | S | T | U | V | W | X | Y | Z | [ | ¥ | ] | ^ | π |
    /// | **6x** | ` | a | b | c | d | e | f | g | h | i | j | k | l | m | n | o |
    /// | **7x** | p | q | r | s | t | u | v | w | x | y | z | { | \| | } | ~ | **DEL** |
    /// | **8x** | │ | ─ | ┴ | ┬ | ┤ | ├ | ┌ | └ | ┐ | ┘ | ╭ | ╰ | ╮ | ╯ | ↑ | ← |
    /// | **9x** | ▒ | ╳ | ┼ | ╱ | ╲ | ◢ | ◣ | ◥ | ◤ | ▁ | ▂ | ▄ | ▀ | 🮂 | ▔ | ▏ |
    /// | **Ax** | Â | Ǎ | Á | À | Ä | Å | Ã | Ā | Ê | Ě | Ë | Ē | É | È | Ñ | N̂ |
    /// | **Bx** | Ǐ | Ì | Í | Ï | Î | Ī | Ô | Ǒ | O̧ | Ó | Ò | Ö | Õ | Û | Ǔ | Ú |
    /// | **Cx** | Ù | Ü | Ū | α | β | θ | λ | μ | Σ | Φ | Ω | Ç | ¿ | ¡ | *A* | £ |
    /// | **Dx** |
    /// | **Ex** | ▎ | ▌ | ▐ | 🮇 | ▕ | █ | | | | | ▞ | ○ | ● |
    /// | **Fx** | | | | | | ♠ | ♥ | ♦ | ♣ | | | | | 🯅 | ÷ |
    ///
    /// Note:
    /// - 0x5c is `¥` instead of `\`, and 0x5f is `π` instead of `_`, as in standard ASCII
    /// - 0xaf and 0xb8 require the use of combining characters in Unicode
    ///   because no single code point exists for the equivalent characters
    /// - 0xce is a combining character used before an 'E' to achieve the
    ///   appearance of an 'Æ' character
    Export,
}

impl CharacterSet {
    /// Try to guess the best character set of a unicode string
    pub fn guess(source: &str) -> Result<Self> {
        let (j_count, e_count) =
            source
                .nfc()
                .collect::<String>()
                .chars()
                .fold((0, 0), |(j, e), src_char| {
                    (
                        if JAPANESE_REVERSE_CHARMAP.contains_key(&src_char) {
                            j + 1
                        } else {
                            j
                        },
                        if EXPORT_REVERSE_CHARMAP.contains_key(&src_char) {
                            e + 1
                        } else {
                            e
                        },
                    )
                });

        if (j_count == 0) && (e_count == 0) {
            Err(ConversionError::NonRepresentableCharacterFound)
        } else if j_count > e_count {
            Ok(Self::Japanese)
        } else {
            Ok(Self::Export)
        }
    }

    fn get(&self, byte: &u8) -> Option<&char> {
        match self {
            Self::Japanese => JAPANESE_CHARMAP.get(byte),
            Self::Export => EXPORT_CHARMAP.get(byte),
        }
    }

    fn get_reverse(&self, chr: &char) -> Option<&u8> {
        match self {
            Self::Japanese => JAPANESE_REVERSE_CHARMAP.get(chr),
            Self::Export => EXPORT_REVERSE_CHARMAP.get(chr),
        }
    }
}

/// A struct for holding a string of text from an SC-3000
#[derive(Clone, Debug)]
pub struct SC3000String {
    /// Character set of this string
    pub cset: CharacterSet,

    /// The bytes containg the string data
    pub bytes: Box<[u8]>,
}

impl SC3000String {
    /// Constructor from bytes and a [CharacterSet]
    pub fn from_bytes(bytes: &[u8], cset: CharacterSet) -> Self {
        Self {
            cset,
            bytes: bytes.into(),
        }
    }

    const N_CIRCUMFLEX_CODE: u8 = 0xaf;
    const N_CIRCUMFLEX_STR: &str = "\u{0302}N";
    const O_CEDILLA_CODE: u8 = 0xb8;
    const O_CEDILLA_STR: &str = "\u{0327}O";
    const AE_HALF_A_CODE: u8 = 0xce;
    const AE_CHAR: char = '\u{00c6}';

    /// Constructor from a Unicode string and a [CharacterSet]
    pub fn from_string(source: &str, cset: CharacterSet) -> Self {
        let source = source.nfc().collect::<String>();
        let mut bytes: Vec<u8> = Vec::with_capacity(source.len());

        let mut i = 0;
        while i < source.len() {
            let src_char = source[i..].chars().next().unwrap();
            if cset == CharacterSet::Export {
                // Handle 0xaf using a combining character
                if source[i..].starts_with(Self::N_CIRCUMFLEX_STR) {
                    bytes.push(Self::N_CIRCUMFLEX_CODE);
                    i += Self::N_CIRCUMFLEX_STR.len();
                    continue;
                }
                // Handle 0xb8 using a combining character
                if source[i..].starts_with(Self::O_CEDILLA_STR) {
                    bytes.push(Self::O_CEDILLA_CODE);
                    i += Self::O_CEDILLA_STR.len();
                    continue;
                }
                // Handle Æ
                if src_char == Self::AE_CHAR {
                    bytes.push(Self::AE_HALF_A_CODE); // "Combining half-A for Æ"
                    bytes.push(b'E');
                    i += Self::AE_CHAR.len_utf8();
                    continue;
                }
            }
            if let Some(b) = cset.get_reverse(&src_char) {
                bytes.push(*b);
                i += 1;
                while !source.is_char_boundary(i) && i < source.len() {
                    i += 1;
                }
                continue;
            }

            // TODO: what to do with unmapped characters?
            i += 1;
            while !source.is_char_boundary(i) && i < source.len() {
                i += 1;
            }
        }

        bytes.shrink_to_fit();
        Self {
            cset,
            bytes: bytes.into(),
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.len() == 0
    }
}

impl core::fmt::Display for SC3000String {
    /// Convert contained bytes into Unicode, write to the output formatter
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut i = 0;
        while i < self.bytes.len() {
            let src_byte = self.bytes[i];
            if self.cset == CharacterSet::Export {
                // Character 0xaf is handled with a combining character
                if src_byte == Self::N_CIRCUMFLEX_CODE {
                    write!(f, "{}", Self::N_CIRCUMFLEX_STR)?;
                    i += 1;
                    continue;
                }
                // Character 0xb8 is handled with a combining character
                if src_byte == Self::O_CEDILLA_CODE {
                    write!(f, "{}", Self::O_CEDILLA_STR)?;
                    i += 1;
                    continue;
                }
                // Handle 0xce "Combining half-A for Æ" followed by 'E'
                if src_byte == Self::AE_HALF_A_CODE
                    && i + 1 < self.bytes.len()
                    && self.bytes[i + 1] == b'E'
                {
                    write!(f, "{}", Self::AE_CHAR)?;
                    i += 2;
                    continue;
                }
            }
            // Characters in the charmap result in the value
            if let Some(c) = self.cset.get(&src_byte) {
                write!(f, "{}", *c)?;
                i += 1;
                continue;
            }

            // Unused and invalid characters result in REPLACEMENT CHARACTER
            write!(f, "\u{fffd}")?;
            i += 1;
        }

        Ok(())
    }
}

impl From<(&[u8], CharacterSet)> for SC3000String {
    fn from((bytes, cset): (&[u8], CharacterSet)) -> Self {
        Self {
            cset,
            bytes: bytes.into(),
        }
    }
}

impl<const N: usize> From<([u8; N], CharacterSet)> for SC3000String {
    fn from((bytes, cset): ([u8; N], CharacterSet)) -> Self {
        Self {
            cset,
            bytes: bytes.into(),
        }
    }
}

impl<const N: usize> From<(&[u8; N], CharacterSet)> for SC3000String {
    fn from((bytes, cset): (&[u8; N], CharacterSet)) -> Self {
        Self {
            cset,
            bytes: (*bytes).into(),
        }
    }
}

impl TryFrom<&str> for SC3000String {
    type Error = ConversionError;

    /// Try to make an SC3000String from a Unicode str, guessing the character set
    fn try_from(t: &str) -> Result<Self> {
        let cs = CharacterSet::guess(t)?;
        Ok(Self::from_string(t, cs))
    }
}

impl TryFrom<&String> for SC3000String {
    type Error = ConversionError;

    /// Try to make an SC3000String from a Unicode String, guessing the character set
    fn try_from(t: &String) -> Result<Self> {
        let cs = CharacterSet::guess(t)?;
        Ok(Self::from_string(t, cs))
    }
}

impl<const N: usize> TryInto<[u8; N]> for SC3000String {
    type Error = ConversionError;

    fn try_into(self) -> Result<[u8; N]> {
        if N > self.len() {
            let mut x: [u8; N] = [0xa0; N];
            x[..self.len()].copy_from_slice(&self.bytes[..self.len()]);
            Ok(x)
        } else {
            Err(ConversionError::BufferTooSmall)
        }
    }
}

impl From<SC3000String> for String {
    fn from(sc3kstr: SC3000String) -> Self {
        sc3kstr.to_string()
    }
}

// List of Japanese characters and their substitutions
static JAPANESE_CHARLIST: [(u8, char); 216] = [
    (0x0d, '\x0a'),

    (0x20, ' '), (0x21, '!'), (0x22, '"'), (0x23, '#'),
    (0x24, '$'), (0x25, '%'), (0x26, '&'), (0x27, '\''),
    (0x28, '('), (0x29, ')'), (0x2a, '*'), (0x2b, '+'),
    (0x2c, ','), (0x2d, '-'), (0x2e, '.'), (0x2f, '/'),
    (0x30, '0'), (0x31, '1'), (0x32, '2'), (0x33, '3'),
    (0x34, '4'), (0x35, '5'), (0x36, '6'), (0x37, '7'),
    (0x38, '8'), (0x39, '9'), (0x3a, ':'), (0x3b, ';'),
    (0x3c, '<'), (0x3d, '='), (0x3e, '>'), (0x3f, '?'),
    (0x40, '@'), (0x41, 'A'), (0x42, 'B'), (0x43, 'C'),
    (0x44, 'D'), (0x45, 'E'), (0x46, 'F'), (0x47, 'G'),
    (0x48, 'H'), (0x49, 'I'), (0x4a, 'J'), (0x4b, 'K'),
    (0x4c, 'L'), (0x4d, 'M'), (0x4e, 'N'), (0x4f, 'O'),
    (0x50, 'P'), (0x51, 'Q'), (0x52, 'R'), (0x53, 'S'),
    (0x54, 'T'), (0x55, 'U'), (0x56, 'V'), (0x57, 'W'),
    (0x58, 'X'), (0x59, 'Y'), (0x5a, 'Z'), (0x5b, '['),
    (0x5c, '\u{00A5}'), (0x5d, '['), (0x5e, '^'), (0x5f, '\u{03C0}'),
    (0x60, '`'), (0x61, 'a'), (0x62, 'b'), (0x63, 'c'),
    (0x64, 'd'), (0x65, 'e'), (0x66, 'f'), (0x67, 'g'),
    (0x68, 'h'), (0x69, 'i'), (0x6a, 'j'), (0x6b, 'k'),
    (0x6c, 'l'), (0x6d, 'm'), (0x6e, 'n'), (0x6f, 'o'),
    (0x70, 'p'), (0x71, 'q'), (0x72, 'r'), (0x73, 's'),
    (0x74, 't'), (0x75, 'u'), (0x76, 'v'), (0x77, 'w'),
    (0x78, 'x'), (0x79, 'y'), (0x7a, 'z'), (0x7b, '{'),
    (0x7c, '|'), (0x7d, '}'), (0x7e, '~'), (0x7f, '\x7f'),
    (0x80, '\u{2502}'), (0x81, '\u{2500}'), (0x82, '\u{2534}'), (0x83, '\u{252c}'),
    (0x84, '\u{2524}'), (0x85, '\u{251c}'), (0x86, '\u{250c}'), (0x87, '\u{2514}'),
    (0x88, '\u{2510}'), (0x89, '\u{2518}'), (0x8a, '\u{256d}'), (0x8b, '\u{2570}'),
    (0x8c, '\u{256e}'), (0x8d, '\u{256f}'), (0x8e, '\u{2191}'), (0x8f, '\u{2190}'),
    (0x90, '\u{2592}'), (0x91, '\u{2573}'), (0x92, '\u{253c}'), (0x93, '\u{2571}'),
    (0x94, '\u{2572}'), (0x95, '\u{25e2}'), (0x96, '\u{25e3}'), (0x97, '\u{25e5}'),
    (0x98, '\u{25e4}'), (0x99, '\u{2581}'), (0x9a, '\u{2582}'), (0x9b, '\u{2584}'),
    (0x9c, '\u{2580}'), (0x9d, '\u{01fb82}'), (0x9e, '\u{2594}'), (0x9f, '\u{258f}'),
    (0xa0, '\u{00a0}'), (0xa1, '\u{3002}'), (0xa2, '\u{300c}'), (0xa3, '\u{300d}'),
    (0xa4, '\u{3001}'), (0xa5, '\u{30fb}'), (0xa6, '\u{30f2}'), (0xa7, '\u{30a1}'),
    (0xa8, '\u{30a3}'), (0xa9, '\u{30a5}'), (0xaa, '\u{30a7}'), (0xab, '\u{30a9}'),
    (0xac, '\u{30e3}'), (0xad, '\u{30e5}'), (0xae, '\u{30e7}'), (0xaf, '\u{30c3}'),
    (0xb0, '\u{30fc}'), (0xb1, '\u{30a2}'), (0xb2, '\u{30a4}'), (0xb3, '\u{30a6}'),
    (0xb4, '\u{30a8}'), (0xb5, '\u{30aa}'), (0xb6, '\u{30ab}'), (0xb7, '\u{30ad}'),
    (0xb8, '\u{30af}'), (0xb9, '\u{30b1}'), (0xba, '\u{30b3}'), (0xbb, '\u{30b5}'),
    (0xbc, '\u{30b7}'), (0xbd, '\u{30b9}'), (0xbe, '\u{30bb}'), (0xbf, '\u{30bd}'),
    (0xc0, '\u{30bf}'), (0xc1, '\u{30c1}'), (0xc2, '\u{30c4}'), (0xc3, '\u{30c6}'),
    (0xc4, '\u{30c8}'), (0xc5, '\u{30ca}'), (0xc6, '\u{30cb}'), (0xc7, '\u{30cc}'),
    (0xc8, '\u{30cd}'), (0xc9, '\u{30ce}'), (0xca, '\u{30cf}'), (0xcb, '\u{30d2}'),
    (0xcc, '\u{30d5}'), (0xcd, '\u{30d8}'), (0xce, '\u{30db}'), (0xcf, '\u{30de}'),
    (0xd0, '\u{30df}'), (0xd1, '\u{30e0}'), (0xd2, '\u{30e1}'), (0xd3, '\u{30e2}'),
    (0xd4, '\u{30e4}'), (0xd5, '\u{30e6}'), (0xd6, '\u{30e8}'), (0xd7, '\u{30e9}'),
    (0xd8, '\u{30ea}'), (0xd9, '\u{30eb}'), (0xda, '\u{30ec}'), (0xdb, '\u{30ed}'),
    (0xdc, '\u{30ef}'), (0xdd, '\u{30f3}'), (0xde, '\u{309b}'), (0xdf, '\u{309c}'),
    (0xe0, '\u{258e}'), (0xe1, '\u{258c}'), (0xe2, '\u{2590}'), (0xe3, '\u{01fb87}'),
    (0xe4, '\u{2595}'), (0xe5, '\u{2588}'),
    (0xea, '\u{259e}'), (0xeb, '\u{25cb}'),
    (0xec, '\u{25cf}'), (0xed, '\u{5e74}'), (0xee, '\u{6708}'), (0xef, '\u{65e5}'),
    (0xf0, '\u{706b}'), (0xf1, '\u{6c34}'), (0xf2, '\u{6728}'), (0xf3, '\u{91d1}'),
    (0xf4, '\u{571f}'), (0xf5, '\u{1660}'), (0xf6, '\u{2665}'), (0xf7, '\u{2666}'),
    (0xf8, '\u{2663}'),
    (0xfd, '\u{01fbc5}'), (0xfe, '\u{00f7}'),
];

// List of 'Export' characters and their substitutions
static EXPORT_CHARLIST: [(u8, char); 189] = [
    (0x0d, '\x0a'),

    (0x20, ' '), (0x21, '!'), (0x22, '"'), (0x23, '#'),
    (0x24, '$'), (0x25, '%'), (0x26, '&'), (0x27, '\''),
    (0x28, '('), (0x29, ')'), (0x2a, '*'), (0x2b, '+'),
    (0x2c, ','), (0x2d, '-'), (0x2e, '.'), (0x2f, '/'),
    (0x30, '0'), (0x31, '1'), (0x32, '2'), (0x33, '3'),
    (0x34, '4'), (0x35, '5'), (0x36, '6'), (0x37, '7'),
    (0x38, '8'), (0x39, '9'), (0x3a, ':'), (0x3b, ';'),
    (0x3c, '<'), (0x3d, '='), (0x3e, '>'), (0x3f, '?'),
    (0x40, '@'), (0x41, 'A'), (0x42, 'B'), (0x43, 'C'),
    (0x44, 'D'), (0x45, 'E'), (0x46, 'F'), (0x47, 'G'),
    (0x48, 'H'), (0x49, 'I'), (0x4a, 'J'), (0x4b, 'K'),
    (0x4c, 'L'), (0x4d, 'M'), (0x4e, 'N'), (0x4f, 'O'),
    (0x50, 'P'), (0x51, 'Q'), (0x52, 'R'), (0x53, 'S'),
    (0x54, 'T'), (0x55, 'U'), (0x56, 'V'), (0x57, 'W'),
    (0x58, 'X'), (0x59, 'Y'), (0x5a, 'Z'), (0x5b, '['),
    (0x5c, '\u{00A5}'), (0x5d, '['), (0x5e, '^'), (0x5f, '\u{03C0}'),
    (0x60, '`'), (0x61, 'a'), (0x62, 'b'), (0x63, 'c'),
    (0x64, 'd'), (0x65, 'e'), (0x66, 'f'), (0x67, 'g'),
    (0x68, 'h'), (0x69, 'i'), (0x6a, 'j'), (0x6b, 'k'),
    (0x6c, 'l'), (0x6d, 'm'), (0x6e, 'n'), (0x6f, 'o'),
    (0x70, 'p'), (0x71, 'q'), (0x72, 'r'), (0x73, 's'),
    (0x74, 't'), (0x75, 'u'), (0x76, 'v'), (0x77, 'w'),
    (0x78, 'x'), (0x79, 'y'), (0x7a, 'z'), (0x7b, '{'),
    (0x7c, '|'), (0x7d, '}'), (0x7e, '~'), (0x7f, '\x7f'),
    (0x80, '\u{2502}'), (0x81, '\u{2500}'), (0x82, '\u{2534}'), (0x83, '\u{252c}'),
    (0x84, '\u{2524}'), (0x85, '\u{251c}'), (0x86, '\u{250c}'), (0x87, '\u{2514}'),
    (0x88, '\u{2510}'), (0x89, '\u{2518}'), (0x8a, '\u{256d}'), (0x8b, '\u{2570}'),
    (0x8c, '\u{256e}'), (0x8d, '\u{256f}'), (0x8e, '\u{2191}'), (0x8f, '\u{2190}'),
    (0x90, '\u{2592}'), (0x91, '\u{2573}'), (0x92, '\u{253c}'), (0x93, '\u{2571}'),
    (0x94, '\u{2572}'), (0x95, '\u{25e2}'), (0x96, '\u{25e3}'), (0x97, '\u{25e5}'),
    (0x98, '\u{25e4}'), (0x99, '\u{2581}'), (0x9a, '\u{2582}'), (0x9b, '\u{2584}'),
    (0x9c, '\u{2580}'), (0x9d, '\u{01fb82}'), (0x9e, '\u{2594}'), (0x9f, '\u{258f}'),
    (0xa0, '\u{00C2}'), (0xa1, '\u{01CD}'), (0xa2, '\u{00C1}'), (0xa3, '\u{00C0}'),
    (0xa4, '\u{00C4}'), (0xa5, '\u{00C5}'), (0xa6, '\u{00C3}'), (0xa7, '\u{0100}'),
    (0xa8, '\u{00CA}'), (0xa9, '\u{011A}'), (0xaa, '\u{00CB}'), (0xab, '\u{0112}'),
    (0xac, '\u{00C9}'), (0xad, '\u{00C8}'), (0xae, '\u{00D1}'),
    (0xb0, '\u{01cf}'), (0xb1, '\u{00cc}'), (0xb2, '\u{00cd}'), (0xb3, '\u{00cf}'),
    (0xb4, '\u{00ce}'), (0xb5, '\u{012a}'), (0xb6, '\u{00d4}'), (0xb7, '\u{01d1}'),
    (0xb9, '\u{00d3}'), (0xba, '\u{00d2}'), (0xbb, '\u{00d6}'),
    (0xbc, '\u{00d5}'), (0xbd, '\u{00d8}'), (0xbe, '\u{01d3}'), (0xbf, '\u{00da}'),
    (0xc0, '\u{00d9}'), (0xc1, '\u{00dc}'), (0xc2, '\u{016a}'), (0xc3, '\u{03b1}'),
    (0xc4, '\u{03b2}'), (0xc5, '\u{03b8}'), (0xc6, '\u{03bb}'), (0xc7, '\u{03bc}'),
    (0xc8, '\u{03a3}'), (0xc9, '\u{03a6}'), (0xca, '\u{03a9}'), (0xcb, '\u{00c7}'),
    (0xcc, '\u{00bf}'), (0xcd, '\u{00a1}'), (0xcf, '\u{00a3}'),
    (0xe0, '\u{258e}'), (0xe1, '\u{258c}'), (0xe2, '\u{2590}'), (0xe3, '\u{01fb87}'),
    (0xe4, '\u{2595}'), (0xe5, '\u{2588}'),
    (0xea, '\u{259e}'), (0xeb, '\u{25cb}'),
    (0xec, '\u{25cf}'),
    (0xf5, '\u{2660}'), (0xf6, '\u{2665}'), (0xf7, '\u{2666}'),
    (0xf8, '\u{2663}'),
    (0xfd, '\u{01fbc5}'), (0xfe, '\u{00f7}'),
];

lazy_static! {
    static ref JAPANESE_CHARMAP: HashMap<u8, char> = HashMap::from(JAPANESE_CHARLIST);
    static ref JAPANESE_REVERSE_CHARMAP: HashMap<char, u8> =
        JAPANESE_CHARLIST.iter().map(|(k, v)| (*v, *k)).collect();
    static ref EXPORT_CHARMAP: HashMap<u8, char> = HashMap::from(EXPORT_CHARLIST);
    static ref EXPORT_REVERSE_CHARMAP: HashMap<char, u8> =
        EXPORT_CHARLIST.iter().map(|(k, v)| (*v, *k)).collect();
}
