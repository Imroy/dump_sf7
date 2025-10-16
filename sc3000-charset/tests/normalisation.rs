use sc3000_charset::*;

#[test]
fn deu_norm_from_string() {
    assert_eq!(
        SC3000String::from_string("WO\u{0308}rterbuch", CharacterSet::Export).bytes(),
        &[0x57, 0xbb, 0x72, 0x74, 0x65, 0x72, 0x62, 0x75, 0x63, 0x68]
    );
}

#[test]
fn fra_norm_from_string() {
    assert_eq!(
        SC3000String::from_string("FranC\u{0327}ais", CharacterSet::Export).bytes(),
        &[0x46, 0x72, 0x61, 0x6e, 0xcb, 0x61, 0x69, 0x73]
    );
}

