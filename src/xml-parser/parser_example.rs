// Simple struct that is mutated to get next value
pub struct Obj {
    stuff: uint
}

impl<'a> Obj {
    // Necessary mutating function
    pub fn decr(&mut self) {
        self.stuff -= 1;
    }
    #[inline]
    pub fn iterate(&'a mut self) -> ObjIterator<'a> {
        ObjIterator{iter: self}
    }
}

// Struct to help with the Iterator pattern emulating Rust native libraries
pub struct ObjIterator <'b> {
    priv iter: &'b mut Obj
}

// The problem seems to be here
impl<'b> Iterator<uint> for ObjIterator<'b> {
    // Apparently I can't have &'b mut
    fn next(&mut self) -> Option<uint> {
        self.iter.decr();
        // Pointless effects to prevent infinite loop on execution
        if(self.iter.stuff == 0){
            None
        }else {
            Some(self.iter.stuff)
        }
    }
}

fn main() {
    let mut obj = Obj{stuff: 3};
    for c in obj.iterate() {
        println!("Got {}", c);
    }
}