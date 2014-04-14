use std::strbuf::StrBuf;

use xml::util::{is_restricted_char, is_whitespace};
use xml::util::{PopShiftShim, clone_to_str};
#[test]
fn test_restricted(){
    assert_eq!(true, is_restricted_char(&'\x0B'));
    assert_eq!(true, is_restricted_char(&'\x02'));
    assert_eq!(true, is_restricted_char(&'\x0C'));
    assert_eq!(true, is_restricted_char(&'\x0F'));
    assert_eq!(true, is_restricted_char(&'\x1F'));
    assert_eq!(true, is_restricted_char(&'\x7F'));
    assert_eq!(true, is_restricted_char(&'\x84'));
    assert_eq!(true, is_restricted_char(&'\x86'));
    assert_eq!(true, is_restricted_char(&'\x9A'));
    assert_eq!(true, is_restricted_char(&'\U0001FFFE'));
    assert_eq!(true, is_restricted_char(&'\U0001FFFF'));
    assert_eq!(true, is_restricted_char(&'\U0002FFFE'));
    assert_eq!(true, is_restricted_char(&'\U0002FFFF'));
    assert_eq!(true, is_restricted_char(&'\U0003FFFE'));
    assert_eq!(true, is_restricted_char(&'\U0003FFFF'));
    assert_eq!(true, is_restricted_char(&'\U0004FFFE'));
    assert_eq!(true, is_restricted_char(&'\U0004FFFF'));
    assert_eq!(true, is_restricted_char(&'\U0005FFFE'));
    assert_eq!(true, is_restricted_char(&'\U0005FFFF'));
    assert_eq!(true, is_restricted_char(&'\U0006FFFE'));
    assert_eq!(true, is_restricted_char(&'\U0006FFFF'));
    assert_eq!(true, is_restricted_char(&'\U0007FFFE'));
    assert_eq!(true, is_restricted_char(&'\U0007FFFF'));
    assert_eq!(true, is_restricted_char(&'\U0008FFFE'));
    assert_eq!(true, is_restricted_char(&'\U0008FFFF'));
    assert_eq!(true, is_restricted_char(&'\U0009FFFE'));
    assert_eq!(true, is_restricted_char(&'\U0009FFFF'));
    assert_eq!(true, is_restricted_char(&'\U000AFFFE'));
    assert_eq!(true, is_restricted_char(&'\U000AFFFF'));
    assert_eq!(true, is_restricted_char(&'\U000BFFFE'));
    assert_eq!(true, is_restricted_char(&'\U000BFFFF'));
    assert_eq!(true, is_restricted_char(&'\U000CFFFE'));
    assert_eq!(true, is_restricted_char(&'\U000CFFFF'));
    assert_eq!(true, is_restricted_char(&'\U000DFFFE'));
    assert_eq!(true, is_restricted_char(&'\U000DFFFF'));
    assert_eq!(true, is_restricted_char(&'\U000EFFFE'));
    assert_eq!(true, is_restricted_char(&'\U000EFFFF'));
    assert_eq!(true, is_restricted_char(&'\U000FFFFE'));
    assert_eq!(true, is_restricted_char(&'\U000FFFFF'));
}

#[test]
fn test_whitespace(){
    assert!(is_whitespace(&'\x20'));
    assert!(is_whitespace(&'\x09'));
    assert!(is_whitespace(&'\x0D'));
    assert!(is_whitespace(&'\x0A'));
    assert!(!is_whitespace(&'\x0B'));
}
