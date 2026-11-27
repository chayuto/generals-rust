//! A field name ends at `=` as well as at whitespace (`INI::initFromINIMulti` splits
//! with `INI::getSeps()`). Retail data writes `Field= value`.

use zh_common::ini::{IniLoadType, IniReader, SEPS_DEFAULT};

#[test]
fn equals_sign_ends_a_field_name() {
    let text = b"  Spaced = 1\r\n  Left= 2\r\n  Tight=3\r\n  Right =4\r\nEnd\r\n";
    let mut ini = IniReader::new("sep.ini", text, IniLoadType::Overwrite);
    let mut seen = Vec::new();
    ini.read_block_fields(|field, ini| {
        seen.push((field.to_string(), ini.next_token(SEPS_DEFAULT)?.to_string()));
        Ok(())
    })
    .unwrap();
    let expected = [
        ("Spaced", "1"),
        ("Left", "2"),
        ("Tight", "3"),
        ("Right", "4"),
    ];
    assert_eq!(seen.len(), expected.len());
    for ((field, value), (want_field, want_value)) in seen.iter().zip(expected) {
        assert_eq!((field.as_str(), value.as_str()), (want_field, want_value));
    }
}
