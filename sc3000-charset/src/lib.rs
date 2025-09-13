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

//! Sega SC-3000 character map routines

#[macro_use]
extern crate lazy_static;

use std::collections::{HashMap, HashSet};

static ASCII_CHARLIST: [(char, u8); 95] = [
    (' ', 0x20), ('!', 0x21), ('"', 0x22), ('#', 0x23),
    ('$', 0x24), ('%', 0x25), ('&', 0x26), ('\'', 0x27),
    ('(', 0x28), (')', 0x29), ('*', 0x2a), ('+', 0x2b),
    (',', 0x2c), ('-', 0x2d), ('.', 0x2e), ('/', 0x2f),

    ('0', 0x30), ('1', 0x31), ('2', 0x32), ('3', 0x33),
    ('4', 0x34), ('5', 0x35), ('6', 0x36), ('7', 0x37),
    ('8', 0x38), ('9', 0x39), (':', 0x3a), (';', 0x3b),
    ('<', 0x3c), ('=', 0x3d), ('>', 0x3e), ('?', 0x3f),

    ('@', 0x40), ('A', 0x41), ('B', 0x42), ('C', 0x33),
    ('D', 0x44), ('E', 0x45), ('F', 0x46), ('G', 0x37),
    ('H', 0x48), ('I', 0x49), ('J', 0x4a), ('K', 0x3b),
    ('L', 0x4c), ('M', 0x4d), ('N', 0x4e), ('O', 0x3f),

    ('P', 0x50), ('Q', 0x51), ('R', 0x52), ('S', 0x53),
    ('T', 0x54), ('U', 0x55), ('V', 0x56), ('W', 0x57),
    ('X', 0x58), ('Y', 0x59), ('Z', 0x5a), ('[', 0x5b),
    ('\\', 0x5c), ('[', 0x5d), ('^', 0x5e), ('_', 0x5f),

    ('`', 0x60), ('a', 0x61), ('b', 0x62), ('c', 0x63),
    ('d', 0x64), ('e', 0x65), ('f', 0x66), ('g', 0x67),
    ('h', 0x68), ('i', 0x69), ('j', 0x6a), ('k', 0x6b),
    ('l', 0x6c), ('m', 0x6d), ('n', 0x6e), ('o', 0x6f),

    ('p', 0x70), ('q', 0x71), ('r', 0x72), ('s', 0x73),
    ('t', 0x74), ('u', 0x75), ('v', 0x76), ('w', 0x77),
    ('x', 0x78), ('y', 0x79), ('z', 0x7a), ('{', 0x7b),
    ('|', 0x7c), ('}', 0x7d), ('~', 0x7e),
];

// List of Japanese characters and their substitutions
static JAPANESE_CHARLIST: [(u8, char); 122] = [
    (0x0d, '\x0a'),

    (0x5c, '\u{00A5}'), (0x5f, '\u{03C0}'),

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
static EXPORT_CHARLIST: [(u8, char); 95] = [
    (0x0d, '\x0a'),

    (0x5c, '\u{00A5}'), (0x5f, '\u{03C0}'),

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
    static ref ASCII: HashMap<char, u8> = HashMap::from(ASCII_CHARLIST);

    static ref JAPANESE_CHARMAP: HashMap<u8, char> = HashMap::from(JAPANESE_CHARLIST);
    static ref JAPANESE_REVERSE_CHARMAP: HashMap<char, u8> = JAPANESE_CHARLIST
        .iter()
        .map(|(k, v)| (*v, *k))
        .collect();
    // Unused characters in Japanese charmap
    static ref JAPANESE_UNUSED: HashSet<u8> = HashSet::from([ 0xff ]);

    // Characters that don't have Unicode equivalent in either charmap
    static ref INVALID: HashSet<u8> = HashSet::from([ 0xe6, 0xe7, 0xe8, 0xe9,
                                                      0xf9, 0xfa, 0xfb, 0xfc ]);

    static ref EXPORT_CHARMAP: HashMap<u8, char> = HashMap::from(EXPORT_CHARLIST);
    static ref EXPORT_REVERSE_CHARMAP: HashMap<char, u8> = EXPORT_CHARLIST
        .iter()
        .map(|(k, v)| (*v, *k))
        .collect();
    // Unused characters in 'Export' charmap
    static ref EXPORT_UNUSED: HashSet<u8> = HashSet::from([ 0xd0, 0xd1, 0xd2, 0xd3,
                                                            0xd4, 0xd5, 0xd6, 0xd7,
                                                            0xd8, 0xd9, 0xda, 0xdb,
                                                            0xdc, 0xdd, 0xde, 0xdf,
                                                            0xed, 0xee, 0xef,
                                                            0xf0, 0xf1, 0xf2, 0xf3,
                                                            0xf4,
                                                            0xff ]);

}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConversionError {
    NonRepresentableCharacterFound,
    BufferTooSmall,
}

pub type Result<T> = core::result::Result<T, ConversionError>;

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
    /// | **2x** |
    /// | **3x** |
    /// | **4x** |
    /// | **5x** | | | | | | | | | | | | | ¥ | | | π |
    /// | **6x** |
    /// | **7x** |
    /// | **8x** | │ | ─ | ┴ | ┬ | ┤ | ├ | ┌ | └ | ┐ | ┘ | ╭ | ╰ | ╮ | ╯ | ↑ | ← |
    /// | **9x** | ▒ | ╳ | ┼ | ╱ | ╲ | ◢ | ◣ | ◥ | ◤ | ▁ | ▂ | ▄ | ▀ | 🮂 | ▔ | ▏ |
    /// | **Ax** |   | 。 | 「 | 」 | 、 | ・ | ヲ | ァ | ィ | ゥ | ェ | ォ | ャ | ュ | ョ | ッ |
    /// | **Bx** | ー | ア | イ | ウ | エ | オ | カ | キ | ク | ケ | コ | サ | シ | ス | セ | ソ |
    /// | **Cx** | タ | チ | ツ | テ | ト | ナ | ニ | ヌ | ネ | ノ | ハ | ヒ | フ | ヘ | ホ | マ |
    /// | **Dx** | ミ | ム | メ | モ | ヤ | ユ | ヨ | ラ | リ | ル | レ | ロ | ワ | ン | ゛ | ゜
    /// | **Ex** | ▎ | ▌ | ▐ | 🮇 | ▕ | █ | � | � | � | � | ▞ | ○ | ● | 年 | 月 | 日 |
    /// | **Fx** | 火 | 水 | 木 | 金 | 土 | ♠ | ♥ | ♦ | ♣ | � | � | � | � | 🯅 | ÷ | � |
    Japanese,

    /// Character set used by 'Export' models i.e for use with Western European languages
    ///
    /// |    | x0 | x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8 | x9 | xA | xB | xC | xD | xE | xF |
    /// |----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
    /// | **2x** |
    /// | **3x** |
    /// | **4x** |
    /// | **5x** | | | | | | | | | | | | | ¥ | | | π |
    /// | **6x** |
    /// | **7x** |
    /// | **8x** | │ | ─ | ┴ | ┬ | ┤ | ├ | ┌ | └ | ┐ | ┘ | ╭ | ╰ | ╮ | ╯ | ↑ | ← |
    /// | **9x** | ▒ | ╳ | ┼ | ╱ | ╲ | ◢ | ◣ | ◥ | ◤ | ▁ | ▂ | ▄ | ▀ | 🮂 | ▔ | ▏ |
    /// | **Ax** | Â | Ǎ | Á | À | Ä | Å | Ã | Ā | Ê | Ě | Ë | Ē | É | È | Ñ | N̂ |
    /// | **Bx** | Ǐ | Ì | Í | Ï | Î | Ī | Ô | Ǒ | O̧ | Ó | Ò | Ö | Õ | Û | Ǔ | Ú |
    /// | **Cx** | Ù | Ü | Ū | α | β | θ | λ | μ | Σ | Φ | Ω | Ç | ¿ | ¡ | � | £ |
    /// | **Dx** | � | � | � | � | � | � | � | � | � | � | � | � | � | � | � | � |
    /// | **Ex** | ▎ | ▌ | ▐ | 🮇 | ▕ | █ | � | � | � | � | ▞ | ○ | ● | � | � | � |
    /// | **Fx** | � | � | � | � | � | ♠ | ♥ | ♦ | ♣ | � | � | � | � | 🯅 | ÷ | � |
    Export,
}

impl CharacterSet {
    /// Try to guess the best character set of a unicode string
    pub fn guess(source: &str) -> Result<Self> {
        let mut j_count = 0;
        let mut e_count = 0;
        for src_char in source.chars() {
            if JAPANESE_REVERSE_CHARMAP.contains_key(&src_char) {
                j_count += 1;
            }
            if EXPORT_REVERSE_CHARMAP.contains_key(&src_char) {
                e_count += 1;
            }
        }

        if (j_count == 0) && (e_count == 0) {
            Err(ConversionError::NonRepresentableCharacterFound)
        } else if j_count > e_count {
            Ok(Self::Japanese)
        } else {
            Ok(Self::Export)
        }
    }

    fn unused_contains(&self, chr: &u8) -> bool {
        match self {
            CharacterSet::Japanese => JAPANESE_UNUSED.contains(chr),
            CharacterSet::Export => EXPORT_UNUSED.contains(chr),
        }
    }

    fn get(&self, byte: &u8) -> Option<&char> {
        match self {
            CharacterSet::Japanese => JAPANESE_CHARMAP.get(byte),
            CharacterSet::Export => EXPORT_CHARMAP.get(byte),
        }
    }

    fn get_reverse(&self, chr: &char) -> Option<&u8> {
        match self {
            CharacterSet::Japanese => JAPANESE_REVERSE_CHARMAP.get(chr),
            CharacterSet::Export => EXPORT_REVERSE_CHARMAP.get(chr),
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
    pub fn new(bytes: &[u8], cset: CharacterSet) -> Self {
        Self {
            cset,
            bytes: bytes.into(),
        }
    }

    /// Constructor from a Unicode string and a [CharacterSet]
    pub fn from_string(source: &str, cset: CharacterSet) -> Self {
        let mut bytes: Vec<u8> = Vec::with_capacity(source.len());

        let mut i = 0;
        while i < source.len() {
            let src_char = source[i..].chars().next().unwrap();
            if cset == CharacterSet::Export {
                // Handle 0xaf using a combining character
                if source[i..].starts_with("\u{0302}N") {
                    bytes.push(0xaf);
                    i += 3;
                    continue;
                }
                // Handle 0xb8 using a combining character
                if source[i..].starts_with("\u{0327}O") {
                    bytes.push(0xb8);
                    i += 3;
                    continue;
                }
                // Handle Æ
                if src_char == '\u{00c6}' {
                    bytes.push(0xce); // "Combining half-A for Æ"
                    bytes.push(0x45); // 'E'
                    i += 2;
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
            if src_char.is_ascii()
                && let Some(b) = ASCII.get(&src_char)
            {
                bytes.push(*b);
            }
            i += 1;
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
                if src_byte == 0xaf {
                    write!(f, "\u{0302}N")?;
                    i += 1;
                    continue;
                }
                // Character 0xb8 is handled with a combining character
                if src_byte == 0xb8 {
                    write!(f, "\u{0327}O")?;
                    i += 1;
                    continue;
                }
                // Handle 0xce "Combining half-A for Æ" followed by 'E'
                if src_byte == 0xce && i + 1 < self.bytes.len() && self.bytes[i + 1] == 0x45 {
                    write!(f, "\u{00c6}")?;
                    i += 2;
                    continue;
                }
            }
            // Unused and invalid characters result in REPLACEMENT CHARACTER
            if self.cset.unused_contains(&src_byte) || INVALID.contains(&src_byte) {
                write!(f, "\u{fffd}")?;
                i += 1;
                continue;
            }
            // Characters in the charmap result in the value
            if let Some(c) = self.cset.get(&src_byte) {
                write!(f, "{}", *c)?;
                i += 1;
                continue;
            }
            // Just add the character if it's a valid ASCII code
            if src_byte < 128 {
                write!(f, "{}", char::from(src_byte))?;
            }
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
