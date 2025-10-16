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

use sc3000_charset::*;

#[test]
fn ascii_from_bytes() {
    assert_eq!(
        SC3000String::from_bytes(&[0x41, 0x53, 0x43, 0x49, 0x49], CharacterSet::Japanese)
            .to_string(),
        "ASCII"
    );
    assert_eq!(
        SC3000String::from_bytes(&[0x41, 0x53, 0x43, 0x49, 0x49], CharacterSet::Export).to_string(),
        "ASCII"
    );
}

#[test]
fn jap_from_bytes() {
    assert_eq!(
        SC3000String::from_bytes(&[0xc6, 0xce, 0xdd], CharacterSet::Japanese).to_string(),
        "ニホン"
    );
}

#[test]
fn eng_from_bytes() {
    assert_eq!(
        SC3000String::from_bytes(
            &[0x45, 0x6e, 0x67, 0x6c, 0x69, 0x73, 0x68],
            CharacterSet::Export
        )
        .to_string(),
        "English"
    );
}

#[test]
fn deu_from_bytes() {
    assert_eq!(
        SC3000String::from_bytes(
            &[0x57, 0xbb, 0x72, 0x74, 0x65, 0x72, 0x62, 0x75, 0x63, 0x68],
            CharacterSet::Export
        )
        .to_string(),
        "WÖrterbuch"
    );
}

#[test]
fn fra_from_bytes() {
    assert_eq!(
        SC3000String::from_bytes(
            &[0x46, 0x72, 0x61, 0x6e, 0xcb, 0x61, 0x69, 0x73],
            CharacterSet::Export
        )
        .to_string(),
        "FranÇais"
    );
}

#[test]
fn newline_from_bytes() {
    assert_eq!(
        SC3000String::from_bytes(
            &[0x0d],
            CharacterSet::Export
        )
        .to_string(),
        "\n"
    );
}
