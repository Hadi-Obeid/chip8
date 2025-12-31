use crate::Chip8;
use std::{fs, io, path::Path};

static START_ADDRESS: u16 = 0x200;

impl Chip8 {
    pub fn load_rom<P>(&mut self, path: P) -> Result<(), io::Error> 
    where P: AsRef<Path> {
        let data = fs::read(path)?;
        Ok(())
    }


}