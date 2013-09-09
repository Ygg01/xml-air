use std::io::{BytesReader,Reader, ReaderUtil};

pub struct ReadSource<'self> {
    source: @Reader,
    priv do_nothing: &'self str
}

impl<'self> ReadSource<'self> {
/*
    fn from_str(data: ~str) -> ReadSource<'self>{
        let r = @BytesReader {
                bytes : data.as_bytes(),
                pos:  @mut 0
        } as @Reader;

        ReadSource {
            source :  r, 
            do_nothing : ""
        }
    }*/

    fn next(&mut self){
        self.source.read_char();
    }
}

fn main() {
    let r = @BytesReader {
                bytes : "as".as_bytes(),
                pos: @mut 0
    } as @Reader;

    let rs = ReadSource {
        source : r, do_nothing : ""
    };

    rs.source.read_char();
}