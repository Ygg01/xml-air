use std::io::{with_str_reader,Reader};

pub struct ReadSource<'self> {
    priv source: &'self Reader
}

impl<'self> ReadSource<'self> {

    fn from_str(data: &'self str) -> ReadSource<'self>{
        let r = std::io::with_str_reader(data, | reader| { reader });
        ReadSource{
            source : r
        }
    }
}

fn main() {

    
}