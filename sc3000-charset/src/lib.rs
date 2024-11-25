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
//!
//! Using data from [Wikipedia](https://en.wikipedia.org/wiki/Sega_SC-3000_character_set) and
//! [SMS Power!](https://www.smspower.org/Development/SC-3000Font).
//! Characters listed as 'unused' or which cannot be represented will be replaced with
//! [**U+FFFD REPLACEMENT CHARACTER**](https://en.wikipedia.org/wiki/Specials_(Unicode_block)#Replacement_character).
//!
//! ## Japanese character set
//!
//! |    | x0 | x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8 | x9 | xA | xB | xC | xD | xE | xF |
//! |----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
//! | **2x** |
//! | **3x** |
//! | **4x** |
//! | **5x** | | | | | | | | | | | | | ¥ | | | π |
//! | **6x** |
//! | **7x** |
//! | **8x** | │ | ─ | ┴ | ┬ | ┤ | ├ | ┌ | └ | ┐ | ┘ | ╭ | ╰ | ╮ | ╯ | ↑ | ← |
//! | **9x** | ▒ | ╳ | ┼ | ╱ | ╲ | ◢ | ◣ | ◥ | ◤ | ▁ | ▂ | ▄ | ▀ | 🮂 | ▔ | ▏ |
//! | **Ax** |   | 。 | 「 | 」 | 、 | ・ | ヲ | ァ | ィ | ゥ | ェ | ォ | ャ | ュ | ョ | ッ |
//! | **Bx** | ー | ア | イ | ウ | エ | オ | カ | キ | ク | ケ | コ | サ | シ | ス | セ | ソ |
//! | **Cx** | タ | チ | ツ | テ | ト | ナ | ニ | ヌ | ネ | ノ | ハ | ヒ | フ | ヘ | ホ | マ |
//! | **Dx** | ミ | ム | メ | モ | ヤ | ユ | ヨ | ラ | リ | ル | レ | ロ | ワ | ン | ゛ | ゜
//! | **Ex** | ▎ | ▌ | ▐ | 🮇 | ▕ | █ | � | � | � | � | ▞ | ○ | ● | 年 | 月 | 日 |
//! | **Fx** | 火 | 水 | 木 | 金 | 土 | ♠ | ♥ | ♦ | ♣ | � | � | � | � | 🯅 | ÷ | � |
//!
//! ## 'Export' character set
//!
//! |    | x0 | x1 | x2 | x3 | x4 | x5 | x6 | x7 | x8 | x9 | xA | xB | xC | xD | xE | xF |
//! |----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
//! | **2x** |
//! | **3x** |
//! | **4x** |
//! | **5x** | | | | | | | | | | | | | ¥ | | | π |
//! | **6x** |
//! | **7x** |
//! | **8x** | │ | ─ | ┴ | ┬ | ┤ | ├ | ┌ | └ | ┐ | ┘ | ╭ | ╰ | ╮ | ╯ | ↑ | ← |
//! | **9x** | ▒ | ╳ | ┼ | ╱ | ╲ | ◢ | ◣ | ◥ | ◤ | ▁ | ▂ | ▄ | ▀ | 🮂 | ▔ | ▏ |
//! | **Ax** | Â | Ǎ | Á | À | Ä | Å | Ã | Ā | Ê | Ě | Ë | Ē | É | È | Ñ | N̂ |
//! | **Bx** | Ǐ | Ì | Í | Ï | Î | Ī | Ô | Ǒ | O̧ | Ó | Ò | Ö | Õ | Û | Ǔ | Ú |
//! | **Cx** | Ù | Ü | Ū | α | β | θ | λ | μ | Σ | Φ | Ω | Ç | ¿ | ¡ | � | £ |
//! | **Dx** | � | � | � | � | � | � | � | � | � | � | � | � | � | � | � | � |
//! | **Ex** | ▎ | ▌ | ▐ | 🮇 | ▕ | █ | � | � | � | � | ▞ | ○ | ● | � | � | � |
//! | **Fx** | � | � | � | � | � | ♠ | ♥ | ♦ | ♣ | � | � | � | � | 🯅 | ÷ | � |

#![feature(ascii_char)]

#[macro_use]
extern crate lazy_static;

use std::collections::{HashMap,HashSet};

static JAPANESE_CHARLIST: [(u8, char); 122] = [
    ( 0x0d, '\x0a' ),

    ( 0x5c, '\u{00A5}' ), ( 0x5f, '\u{03C0}' ),

    ( 0x80, '\u{2502}' ), ( 0x81, '\u{2500}' ), ( 0x82, '\u{2534}' ), ( 0x83, '\u{252c}' ),
    ( 0x84, '\u{2524}' ), ( 0x85, '\u{251c}' ), ( 0x86, '\u{250c}' ), ( 0x87, '\u{2514}' ),
    ( 0x88, '\u{2510}' ), ( 0x89, '\u{2518}' ), ( 0x8a, '\u{256d}' ), ( 0x8b, '\u{2570}' ),
    ( 0x8c, '\u{256e}' ), ( 0x8d, '\u{256f}' ), ( 0x8e, '\u{2191}' ), ( 0x8f, '\u{2190}' ),

    ( 0x90, '\u{2592}' ), ( 0x91, '\u{2573}' ), ( 0x92, '\u{253c}' ), ( 0x93, '\u{2571}' ),
    ( 0x94, '\u{2572}' ), ( 0x95, '\u{25e2}' ), ( 0x96, '\u{25e3}' ), ( 0x97, '\u{25e5}' ),
    ( 0x98, '\u{25e4}' ), ( 0x99, '\u{2581}' ), ( 0x9a, '\u{2582}' ), ( 0x9b, '\u{2584}' ),
    ( 0x9c, '\u{2580}' ), ( 0x9d, '\u{01fb82}' ), ( 0x9e, '\u{2594}' ), ( 0x9f, '\u{258f}' ),

    ( 0xa0, '\u{00a0}' ), ( 0xa1, '\u{3002}' ), ( 0xa2, '\u{300c}' ), ( 0xa3, '\u{300d}' ),
    ( 0xa4, '\u{3001}' ), ( 0xa5, '\u{30fb}' ), ( 0xa6, '\u{30f2}' ), ( 0xa7, '\u{30a1}' ),
    ( 0xa8, '\u{30a3}' ), ( 0xa9, '\u{30a5}' ), ( 0xaa, '\u{30a7}' ), ( 0xab, '\u{30a9}' ),
    ( 0xac, '\u{30e3}' ), ( 0xad, '\u{30e5}' ), ( 0xae, '\u{30e7}' ), ( 0xaf, '\u{30c3}' ),

    ( 0xb0, '\u{30fc}' ), ( 0xb1, '\u{30a2}' ), ( 0xb2, '\u{30a4}' ), ( 0xb3, '\u{30a6}' ),
    ( 0xb4, '\u{30a8}' ), ( 0xb5, '\u{30aa}' ), ( 0xb6, '\u{30ab}' ), ( 0xb7, '\u{30ad}' ),
    ( 0xb8, '\u{30af}' ), ( 0xb9, '\u{30b1}' ), ( 0xba, '\u{30b3}' ), ( 0xbb, '\u{30b5}' ),
    ( 0xbc, '\u{30b7}' ), ( 0xbd, '\u{30b9}' ), ( 0xbe, '\u{30bb}' ), ( 0xbf, '\u{30bd}' ),

    ( 0xc0, '\u{30bf}' ), ( 0xc1, '\u{30c1}' ), ( 0xc2, '\u{30c4}' ), ( 0xc3, '\u{30c6}' ),
    ( 0xc4, '\u{30c8}' ), ( 0xc5, '\u{30ca}' ), ( 0xc6, '\u{30cb}' ), ( 0xc7, '\u{30cc}' ),
    ( 0xc8, '\u{30cd}' ), ( 0xc9, '\u{30ce}' ), ( 0xca, '\u{30cf}' ), ( 0xcb, '\u{30d2}' ),
    ( 0xcc, '\u{30d5}' ), ( 0xcd, '\u{30d8}' ), ( 0xce, '\u{30db}' ), ( 0xcf, '\u{30de}' ),

    ( 0xd0, '\u{30df}' ), ( 0xd1, '\u{30e0}' ), ( 0xd2, '\u{30e1}' ), ( 0xd3, '\u{30e2}' ),
    ( 0xd4, '\u{30e4}' ), ( 0xd5, '\u{30e6}' ), ( 0xd6, '\u{30e8}' ), ( 0xd7, '\u{30e9}' ),
    ( 0xd8, '\u{30ea}' ), ( 0xd9, '\u{30eb}' ), ( 0xda, '\u{30ec}' ), ( 0xdb, '\u{30ed}' ),
    ( 0xdc, '\u{30ef}' ), ( 0xdd, '\u{30f3}' ), ( 0xde, '\u{309b}' ), ( 0xdf, '\u{309c}' ),

    ( 0xe0, '\u{258e}' ), ( 0xe1, '\u{258c}' ), ( 0xe2, '\u{2590}' ), ( 0xe3, '\u{01fb87}' ),
    ( 0xe4, '\u{2595}' ), ( 0xe5, '\u{2588}' ),
    ( 0xea, '\u{259e}' ), ( 0xeb, '\u{25cb}' ),
    ( 0xec, '\u{25cf}' ), ( 0xed, '\u{5e74}' ), ( 0xee, '\u{6708}' ), ( 0xef, '\u{65e5}' ),

    ( 0xf0, '\u{706b}' ), ( 0xf1, '\u{6c34}' ), ( 0xf2, '\u{6728}' ), ( 0xf3, '\u{91d1}' ),
    ( 0xf4, '\u{571f}' ), ( 0xf5, '\u{1660}' ), ( 0xf6, '\u{2665}' ), ( 0xf7, '\u{2666}' ),
    ( 0xf8, '\u{2663}' ),
    ( 0xfd, '\u{01fbc5}' ), ( 0xfe, '\u{00f7}' ),
];

static EXPORT_CHARLIST: [(u8, char); 97] = [
    ( 0x0d, '\x0a' ),

    ( 0x5c, '\u{00A5}' ), ( 0x5f, '\u{03C0}' ),

    ( 0x80, '\u{2502}' ), ( 0x81, '\u{2500}' ), ( 0x82, '\u{2534}' ), ( 0x83, '\u{252c}' ),
    ( 0x84, '\u{2524}' ), ( 0x85, '\u{251c}' ), ( 0x86, '\u{250c}' ), ( 0x87, '\u{2514}' ),
    ( 0x88, '\u{2510}' ), ( 0x89, '\u{2518}' ), ( 0x8a, '\u{256d}' ), ( 0x8b, '\u{2570}' ),
    ( 0x8c, '\u{256e}' ), ( 0x8d, '\u{256f}' ), ( 0x8e, '\u{2191}' ), ( 0x8f, '\u{2190}' ),

    ( 0x90, '\u{2592}' ), ( 0x91, '\u{2573}' ), ( 0x92, '\u{253c}' ), ( 0x93, '\u{2571}' ),
    ( 0x94, '\u{2572}' ), ( 0x95, '\u{25e2}' ), ( 0x96, '\u{25e3}' ), ( 0x97, '\u{25e5}' ),
    ( 0x98, '\u{25e4}' ), ( 0x99, '\u{2581}' ), ( 0x9a, '\u{2582}' ), ( 0x9b, '\u{2584}' ),
    ( 0x9c, '\u{2580}' ), ( 0x9d, '\u{01fb82}' ), ( 0x9e, '\u{2594}' ), ( 0x9f, '\u{258f}' ),

    ( 0xa0, '\u{00C2}' ), ( 0xa1, '\u{01CD}' ), ( 0xa2, '\u{00C1}' ), ( 0xa3, '\u{00C0}' ),
    ( 0xa4, '\u{00C4}' ), ( 0xa5, '\u{00C5}' ), ( 0xa6, '\u{00C3}' ), ( 0xa7, '\u{0100}' ),
    ( 0xa8, '\u{00CA}' ), ( 0xa9, '\u{011A}' ), ( 0xaa, '\u{00CB}' ), ( 0xab, '\u{0112}' ),
    ( 0xac, '\u{00C9}' ), ( 0xad, '\u{00C8}' ), ( 0xae, '\u{00D1}' ), ( 0xaf, '\u{004E}' ),

    ( 0xb0, '\u{01cf}' ), ( 0xb1, '\u{00cc}' ), ( 0xb2, '\u{00cd}' ), ( 0xb3, '\u{00cf}' ),
    ( 0xb4, '\u{00ce}' ), ( 0xb5, '\u{012a}' ), ( 0xb6, '\u{00d4}' ), ( 0xb7, '\u{01d1}' ),
    ( 0xb8, '\u{004f}' ), ( 0xb9, '\u{00d3}' ), ( 0xba, '\u{00d2}' ), ( 0xbb, '\u{00d6}' ),
    ( 0xbc, '\u{00d5}' ), ( 0xbd, '\u{00d8}' ), ( 0xbe, '\u{01d3}' ), ( 0xbf, '\u{00da}' ),

    ( 0xc0, '\u{00d9}' ), ( 0xc1, '\u{00dc}' ), ( 0xc2, '\u{016a}' ), ( 0xc3, '\u{03b1}' ),
    ( 0xc4, '\u{03b2}' ), ( 0xc5, '\u{03b8}' ), ( 0xc6, '\u{03bb}' ), ( 0xc7, '\u{03bc}' ),
    ( 0xc8, '\u{03a3}' ), ( 0xc9, '\u{03a6}' ), ( 0xca, '\u{03a9}' ), ( 0xcb, '\u{00c7}' ),
    ( 0xcc, '\u{00bf}' ), ( 0xcd, '\u{00a1}' ), ( 0xcf, '\u{00a3}' ),

    ( 0xe0, '\u{258e}' ), ( 0xe1, '\u{258c}' ), ( 0xe2, '\u{2590}' ), ( 0xe3, '\u{01fb87}' ),
    ( 0xe4, '\u{2595}' ), ( 0xe5, '\u{2588}' ),
    ( 0xea, '\u{259e}' ), ( 0xeb, '\u{25cb}' ),
    ( 0xec, '\u{25cf}' ),

    ( 0xf5, '\u{2660}' ), ( 0xf6, '\u{2665}' ), ( 0xf7, '\u{2666}' ),
    ( 0xf8, '\u{2663}' ),
    ( 0xfd, '\u{01fbc5}' ), ( 0xfe, '\u{00f7}' ),
];

lazy_static! {
    static ref JAPANESE_CHARMAP: HashMap<u8, char> = HashMap::from(JAPANESE_CHARLIST);
    // Unused characters in Japanese charmap
    static ref JAPANESE_UNUSED: HashSet<u8> = HashSet::from([ 0xff ]);

    // Characters that don't have Unicode equivalent in either charmap
    static ref INVALID: HashSet<u8> = HashSet::from([ 0xe6, 0xe7, 0xe8, 0xe9,
                                                      0xf9, 0xfa, 0xfb, 0xfc ]);

    static ref EXPORT_CHARMAP: HashMap<u8, char> = HashMap::from(EXPORT_CHARLIST);
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

/// Convert a byte slice of Japanese SC-3000 characters to a Unicode string
pub fn convert_japanese_to_unicode(source: &[u8]) -> String {
    let mut dest = "".to_string();
    dest.reserve(source.len());

    for src_byte in source {
        // Unused and invalid characters result in REPLACEMENT CHARACTER
        if JAPANESE_UNUSED.contains(src_byte) || INVALID.contains(src_byte) {
            dest.push('\u{fffd}');
            continue;
        }
        // Characters in the Japanese charmap result in the value
        if let Some((_, c)) = JAPANESE_CHARMAP.get_key_value(src_byte) {
            dest.push(*c);
            continue;
        }
        // Just add the character if it's a valid ASCII code
        if *src_byte < 128 {
            dest.push(char::from(*src_byte));
        }
    }

    dest.shrink_to_fit();
    dest

}

/// Convert a byte slice of 'Export' SC-3000 characters to a Unicode string
pub fn convert_export_to_unicode(source: &[u8]) -> String {
    let mut dest = "".to_string();
    dest.reserve(source.len());

    let mut src_iter = source.iter().peekable();
    while let Some(&src_byte) = src_iter.next() {
        // Handle 0xce "Combining half-A for Æ" followed by 'E'
        if (src_byte == 0xce) && (src_iter.peek() == Some(&&0x45_u8)) {
            dest.push('\u{00c6}');
            if src_iter.next().is_none() {
                break;
            }
            continue;
        }

        // Unused and invalid characters result in REPLACEMENT CHARACTER
        if EXPORT_UNUSED.contains(&src_byte) || INVALID.contains(&src_byte) {
            dest.push('\u{fffd}');
            continue;
        }
        // Characters in the 'Export' charmap result in the value
        if let Some((_, c)) = EXPORT_CHARMAP.get_key_value(&src_byte) {
            dest.push(*c);
            continue;
        }
        // Just add the character if it's a valid ASCII code
        if src_byte < 128 {
            dest.push(char::from(src_byte));
        }
    }

    dest.shrink_to_fit();
    dest

}
