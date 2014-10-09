pub fn is_hex_digit(c: char) -> bool {
    let c_int = match c {
      '0' ... '9' => c as uint - ('0' as uint),
      'a' ... 'z' => c as uint + 10u - ('a' as uint),
      'A' ... 'Z' => c as uint + 10u - ('A' as uint),
      _ => 42,
    };
    if c_int <= 15 {return true;}
    return false;
}

pub fn is_digit(c: char) -> bool {
    match c {
        '0'...'9' => return true,
        _ => return false,
    }
}
