mod tests;
use std::ops::{Index, IndexMut};
use std::fmt;
use std::io::{self, Write};
use std::char;
use core::mem;

const TOM:usize = 0x8000; // Top Of Memory
const NUM_REG:usize = 8;

// status reg
// x = undefined
// x x x x    OUT MEMR M1 HLT?
const OUT_BIT:u16 = 3;      // IO event
const MEMR_BIT:u16 = 2;     // mem read
const M1_BIT:u16 = 1;       // M1 cycle
const HALT_BIT:u16 = 0;

fn set_bit(data:&mut u16, bit_position:u16) {
    *data |= (1 << bit_position);
}

fn clear_bit(data:&mut u16, bit_position:u16) {
    *data &= (0xFFFF ^ (1 << bit_position));
}

fn get_bit(data:&u16, bit_position:u16) -> bool {
    if (data & (1 << bit_position)) > 0 {
        true
    } else {
        false
    }
}

/**
 * Takes a u16 that's expressed in either little-endian or big-endian and swaps it
 * e.g., 0x00FE returns as 0xFE00, and 0xFE00 returns as 0x00FE
 */
fn swap_endian(ushort:u16) -> u16 {
    (ushort << 8) | (ushort >> 8)
}

struct Machine {
    mem:[u16; TOM],
    stack:Vec<u16>,
    registers:[u16; NUM_REG],
    pc:u16,
    status:u16,
}

struct ErrorMemoryInvalid;
impl fmt::Display for ErrorMemoryInvalid {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid memory access")
    }
}
struct ErrorUnknownOpcode;
impl fmt::Display for ErrorUnknownOpcode {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unknown opcode")
    }
}

impl Index<u16> for Machine {
    type Output = u16;
    fn index(&self, addr: u16) -> &u16 {
        if addr < (TOM as u16) {
            &self.mem[addr as usize]
        } else {
            panic!(ErrorMemoryInvalid);
        }
    }
}

impl IndexMut<u16> for Machine {
    fn index_mut(&mut self, addr: u16) -> &mut Self::Output {
        if addr < (TOM as u16) {
            &mut self.mem[addr as usize]
        } else {
            panic!(ErrorMemoryInvalid);
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

    /**
     * Performs the M1 operation to fetch the opcode from mem[pc], swaps its endian-ness
     * and then executes
     */
    pub fn fetch_and_execute(&mut self) {
        if !self.is_halted() {
        
            set_bit(&mut self.status, M1_BIT);
            set_bit(&mut self.status, MEMR_BIT);
            let instruction:u16 = swap_endian(self.mem[self.pc as usize]);
            self.pc += 1;
            clear_bit(&mut self.status, MEMR_BIT);
            clear_bit(&mut self.status, M1_BIT);
            
            self.execute(instruction);
        }
    }

    pub fn run(&mut self) {
        while !self.is_halted() {
            self.fetch_and_execute();
        }
    }

    pub fn is_halted(&self) -> bool {
        get_bit(&self.status, HALT_BIT)
    }

    fn execute(&mut self, instruction:u16) {
        match instruction {
            0x0000 => self.halt(),      // `halt`
            0x0009 => self.add(),
            0x0013 => self.out(),       // `out`  0d19

            0x0015 => self.nop(),       // `noop` 0d21
            _ => panic!(ErrorUnknownOpcode)
        }
    }

    fn peek(&mut self, address:&mut u16, increment_addr:bool) -> u16 {
        set_bit(&mut self.status, MEMR_BIT);
        let val:u16 = swap_endian(self.mem[*address as usize]);
        clear_bit(&mut self.status, MEMR_BIT);

        if increment_addr {
            *address += 1;
        }
        return val;
    }

    fn poke(&mut self, address:&mut u16, value:u16, increment_addr:bool) {
        clear_bit(&mut self.status, MEMR_BIT);
        self.mem[*address as usize] = swap_endian(value);
        clear_bit(&mut self.status, MEMR_BIT);

        if increment_addr {
            *address += 1;
        }
    }

    /**
     * Stops execution and terminates the program
     */
    fn halt(&mut self) {
        set_bit(&mut self.status, HALT_BIT);
    }

    /**
     * Assign into <a> the sum of <b> and <c> (modulo 0x8000)
     */
    fn add(&mut self) {
    /*
        let dest_p:u16 = self.peek(self.pc);
        self.pc += 1;
        let mut sum:u16 = self.peek(self.pc);
        self.pc += 1;
        sum += self.peek(self.pc);
        */
    }

    /**
     * Writes the character represented by ASCII code <a> to the terminal
     */
    fn out(&mut self) {
        // fetch arg
        let pointer:u16 = self.peek(&mut self.pc, true);

        // obtain value
        //let val:u16 = self.peek(&mut pointer, false);
        let val:u16 = 0;

        // ASCII output
        set_bit(&mut self.status, OUT_BIT);
        print!("{}", (val as u8) as char);
        io::stdout().flush().unwrap();
        clear_bit(&mut self.status, OUT_BIT);
    }

    fn nop(&self) {
    }
}

// see tests.rs
fn main() {
}

