mod tests;

use std::ops::{Index, IndexMut};
use std::fmt;

const TOM:usize = 0x8000; // Top Of Memory
const NUM_REG:usize = 8;

// status reg
// x = undefined
// x x x x    x x x HLT?
const HALT_BIT:u8 = 0;

struct Machine {
    mem:[u16; TOM],
    stack:Vec<u16>,
    registers:[u16; NUM_REG],
    pc:u16,
    status:u8,
}

struct MemInvalidError;
impl fmt::Display for MemInvalidError {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid memory access")
    }
}

impl Index<u16> for Machine {
    type Output = u16;
    fn index(&self, addr: u16) -> &u16 {
        if addr < (TOM as u16) {
            &self.mem[addr as usize]
        } else {
            panic!(MemInvalidError);
        }
    }
}

impl IndexMut<u16> for Machine {
    fn index_mut(&mut self, addr: u16) -> &mut Self::Output {
        if addr < (TOM as u16) {
            &mut self.mem[addr as usize]
        } else {
            panic!(MemInvalidError);
        }
    }
}

impl Machine {
    fn new() -> Self {
        Machine {
            stack: vec![],
            registers: [0; NUM_REG],
            pc: 0,
            mem:[0; TOM],
            status: 0,
        }
    }

    pub fn step(&mut self) {
        if !self.is_halted() {
            self.decode(self.mem[self.pc as usize]);
            self.pc += 1;
        }
    }

    pub fn is_halted(&self) -> bool {
        if (self.status & (1 << HALT_BIT)) == 1 {
            true
        } else {
            false
        }
    }

    fn nop(&self) {
    }


    fn decode(&mut self, instruction:u16) {
        match instruction {
            0x00 => self.halt(),
            _ => self.nop()
        }
    }

    fn halt(&mut self) {
        self.status |= (1 << HALT_BIT);
    }
}

// see tests.rs
fn main() {
}

