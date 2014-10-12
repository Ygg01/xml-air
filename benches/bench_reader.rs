extern crate test;
extern crate xml_air;

use test::Bencher;
use std::io::BufReader;

use xml_air::parser::{XmlReader};


static LINE: &'static str = "test \n reader\r\n";

/// This is our raw data source.  Pretend it's on disk somewhere, and it's
/// too big to load into memory all at once.
pub fn make_pretend_file() -> String {
    let mut result: String = String::new();
    for _ in range(0u, 100) { result.push_str(LINE); }
    result
}
#[inline(always)]
fn always_true(c: char) -> bool {
    true
}

#[bench]
fn xml_reader_throughput(b: &mut test::Bencher) {
    let file = make_pretend_file();
    b.bytes = file.len() as u64;
    b.iter(|| -> () {
        let mut input = BufReader::new(file.as_bytes());
        let mut reader = XmlReader::from_reader(&mut input);

        while !reader.eof {
            reader.read_nchar();
        }
    });
}

#[bench]
fn xml_reader_throughput_all(b: &mut test::Bencher) {
    let file = make_pretend_file();
    b.bytes = file.len() as u64;
    b.iter(|| -> () {
        let mut input = BufReader::new(file.as_bytes());
        let mut reader = XmlReader::from_reader(&mut input);

        reader.read_until(always_true, false);
    });
}


#[bench]
fn reader_throughput(b: &mut test::Bencher) {
    let file = make_pretend_file();
    b.bytes = file.len() as u64;
    b.iter(|| -> () {
        let mut reader = BufReader::new(file.as_bytes());

        while !reader.eof() {
            reader.read_char();
        }
    });
}