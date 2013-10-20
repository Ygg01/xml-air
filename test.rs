use std::io::{BytesReader,Reader, ReaderUtil};

pub struct ReadSource {
    source: ~Reader,

}

impl ReadSource {

    fn next(&mut self){
        self.source.read_char();
    }

}

fn main() {
    let r = ~BytesReader {
                bytes : "as".as_bytes(),
                pos: @mut 0
    } as ~Reader;

    let rs = ReadSource{source: r};

}
