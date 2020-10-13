mod tests;
mod errors;
mod display;

use std::ops::{Index, IndexMut};
use std::io::{self, Write, Read};
use std::char;
use crate::errors::Error::{MemoryInvalid, UnknownOpcode, EmptyStack, FailedToReadLine};
use std::fs::File;
use std::num::Wrapping;
use crate::display::frontpanelRun;
use synacor_cpu::Machine;


// see tests.rs
fn main() -> io::Result<()> {
    let mut f = File::open("challenge.bin")?;

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    let mut m0 = Machine::new();
    let mut val:u16 = 0;
    let mut x:u16 = 0;
    for n in (0..buffer.len()).step_by(2) {
        val = (buffer[n] as u16) << 8;
        val |= buffer[n+1] as u16;
        m0.mem.push(val);
        x+=1;
    }

    // frontpanelRun(&mut m0);

    m0.run();

    Ok(())
}

