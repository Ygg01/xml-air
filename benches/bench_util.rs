extern crate test;
use test::Bencher;

fn is_hex_digit(c: char) -> bool {
    match c {
        '0'...'9'
        | 'a'...'f'
        | 'A'...'F' => return true,
        _ => return false,
    }
}

#[bench]
fn hex_match(b: &mut Bencher) {
    let v = "01234567890abcdfghijklmnoprstuwćđšшђађ";
    b.iter(||{
        for c in v.chars() {
            is_hex_digit(c);
        }
    })
}

fn is_hex_digit_cmp(c: char) -> bool {
    let c_int = match c {
      '0' ... '9' => c as uint - ('0' as uint),
      'a' ... 'z' => c as uint + 10u - ('a' as uint),
      'A' ... 'Z' => c as uint + 10u - ('A' as uint),
      _ => 42,
    };
    if c_int <= 15 {return true;}
    return false;
}

#[bench]
fn hex_match_cmp(b: &mut Bencher) {
    let v = "01234567890abcdfghijklmnoprstuwćđšшђађ";
    b.iter(||{
        for c in v.chars() {
            is_hex_digit_cmp(c);
        }
    })
}


fn is_digit(c: char) -> bool {
    match c {
        '0'...'9' => return true,
        _ => return false,
    }
}

#[bench]
fn digit_match(b: &mut Bencher) {
    let v = "01234567890abcdfghijklmnoprstuwćđšшђађ";
    b.iter(||{
        for c in v.chars() {
            is_digit(c);
        }
    })
}


fn is_digit_std(c: char) -> bool {
    return std::char::is_digit(c);
}

#[bench]
fn digit_match_std(b: &mut Bencher) {
    let v = "01234567890abcdfghijklmnoprstuwćđšшђађ";
    b.iter(||{
        for c in v.chars() {
            is_digit_std(c);
        }
    })
}

fn is_digit_cmp(c: char) -> bool {
    let c_int = c as uint - ('0' as uint);
    if c_int <= 9 {return true;}
    return false;
}

#[bench]
fn digit_match_cmp(b: &mut Bencher) {
    let v = "01234567890abcdfghijklmnoprstuwćđšшђађ";
    b.iter(||{
        for c in v.chars() {
            is_digit_cmp(c);
        }
    })
}











