mod tests;
mod errors;

use std::ops::{Index, IndexMut};
use std::fmt;
use std::io::{self, Write};
use std::char;
use core::mem;
use crate::errors::Error::{MemoryInvalid, UnknownOpcode, EmptyStack};
use std::process::id;

const TOM:usize = 0x8000; // Top Of Memory, exclusive (mem: 0x0000-0x7FFF inclusive)
const NUM_REG:usize = 8;

// status reg
// x = undefined
// x x x OUT    MEMW MEMR M1 HLT?
const OUT_BIT:u16 = 4;      // IO event
const MEMW_BIT:u16 = 3;     // mem read
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
 * Takes a u16 that's expressed in either little-endian or big-endian and returned
 * the swapped version
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

impl Index<u16> for Machine {
    type Output = u16;
    fn index(&self, addr: u16) -> &u16 {
        if addr < (TOM as u16) {
            &self.mem[addr as usize]
        } else {
            panic!(MemoryInvalid);
        }
    }
}

impl IndexMut<u16> for Machine {
    fn index_mut(&mut self, addr: u16) -> &mut Self::Output {
        if addr < (TOM as u16) {
            &mut self.mem[addr as usize]
        } else {
            panic!(MemoryInvalid);
        }
    }
}

impl Machine {
    fn new() -> Self {
        Machine {
            stack: Vec::new(),
            registers: [0; NUM_REG],
            pc: 0,
            mem:[0; TOM],
            status: 0,
        }
    }

    /**
     * Fetches the value at mem[pc], converts it to big-endian, increments the pc
     * and returns the value.
     *
     * Sets and clears the MEMR flag in the status register
     */
    fn peek_inc(&mut self) -> u16 {
        set_bit(&mut self.status, MEMR_BIT);
        let mut val:u16 = self.mem[self.pc as usize];
        clear_bit(&mut self.status, MEMR_BIT);

        self.pc += 1;

        return swap_endian(val);
    }

    /**
     * Gets the memory or register at `destination` (see below), converts it to big-endian, and returns it.
     * If `dest_addr` (big-endian) is `<` `TOM`, then `destination` = `mem[dest_addr]`
     * otherwise `dest_addr` refers to a register 0...7: `TOM, TOM+1, ... TOM+7 = registers[0...7]`
     *
     * Sets and clears the `MEMR` flag in the status register
     */
    fn peek(&mut self, dest_addr:u16) -> u16 {
        set_bit(&mut self.status, MEMR_BIT);
        let mut val:u16 = 0;
        if dest_addr < TOM as u16 {
            val = self.mem[dest_addr as usize];
        } else if dest_addr < (TOM+8) as u16 {
            val = self.registers[(dest_addr % (TOM as u16)) as usize];
        } else {
            panic!(MemoryInvalid);
        }
        clear_bit(&mut self.status, MEMR_BIT);

        return swap_endian(val);
    }

    /**
     * Sets the memory or register at `destination` to `value`
     * `value` should be provided big-endian, and it will be converted to
     * little-endian
     * If `dest_addr` is `<` `TOM`, then `destination` = `mem[dest_addr]`
     * otherwise `dest_addr` refers to a register 0...7: `TOM, TOM+1, ... TOM+7 = registers[0...7]`
     *
     * Sets and clears the `MEMW` flag in the status register
     */
    fn poke(&mut self, dest_addr:u16, value:u16) {
        set_bit(&mut self.status, MEMW_BIT);
        if dest_addr < TOM as u16 {
            self.mem[dest_addr as usize] = swap_endian(value);
        } else if dest_addr <= (TOM+7) as u16 {
            self.registers[(dest_addr % (TOM as u16)) as usize] = swap_endian(value);
        } else {
            panic!(MemoryInvalid);
        }
        clear_bit(&mut self.status, MEMW_BIT);
    }

    /**
     * Performs the M1 operation to fetch the opcode from mem[pc], swaps its endian-ness
     * (from LE to BE) and then executes. The fetch increments `pc`
     */
    pub fn fetch_and_execute(&mut self) {
        if !self.is_halted() {
            set_bit(&mut self.status, M1_BIT);
            let instruction:u16 = self.peek_inc();
            clear_bit(&mut self.status, M1_BIT);
            
            self.execute(instruction);
        }
    }

    /**
     * Starts CPU execution at `pc` and continues until `HLT` is set in the status register
     */
    pub fn run(&mut self) {
        while !self.is_halted() {
            self.fetch_and_execute();
        }
    }

    /**
     * Returns `true` if the CPU is halted. `false` otherwise
     */
    pub fn is_halted(&self) -> bool {
        get_bit(&self.status, HALT_BIT)
    }

    /**
     * Given `instruction`, calls the appropriate function to execute the opcode.
     * If `instruction` can't decode into a known instruction, an `ErrorUnknownOpcode`
     * panic occurs.
     */
    fn execute(&mut self, instruction:u16) {
        match instruction {
            0x0000 => self.halt(),      // `halt`
            0x0001 => self.set(),
            0x0002 => self.push(),
            0x0003 => self.pop(),
            0x0004 => self.eq(),
            0x0005 => self.gt(),
            0x0006 => self.jmp(),
            0x0007 => self.jt(),
            0x0008 => self.jf(),
            0x0009 => self.add(),
            0x000A => self.mult(),
            0x000B => self.modulo(),
            0x000C => self.and(),
            0x000D => self.or(),
            0x000E => self.not(),

            0x0013 => self.out(),

            0x0015 => self.nop(),       // `noop` 0d21
            _ => panic!(UnknownOpcode)
        }
    }

    /**
     * Stops execution and terminates the program
     */
    fn halt(&mut self) {
        set_bit(&mut self.status, HALT_BIT);
    }

    /**
     * Set register a to the immediate value of b
     */
    fn set(&mut self) {
        let dest:u16 = self.peek_inc();
        let value:u16 = self.peek_inc();

        self.poke(dest, value);
    }

    /**
     * Pushes immediate value a onto the stack
     */
    fn push(&mut self) {
        let value:u16 = self.peek_inc();
        self.stack.push(value);
    }
   

    /**
     * Remove the top element from the stack and write it into a
     * An empty stack panics
     */
    fn pop(&mut self) {
        let value:u16 = match self.stack.pop() {
            Some(p) => p,
            None => panic!(EmptyStack)
        };
        let dest:u16 = self.peek_inc();
        self.poke(dest, value);
    }

    /**
     * set a to 1 if b is equal than c; set it to 0 otherwise
     */
    fn eq(&mut self) {
        let dest:u16 = self.peek_inc();
        let b:u16 = self.peek_inc();
        let c:u16 = self.peek_inc();

        if b == c {
            self.poke(dest, 1);
        } else {
            self.poke(dest, 0);
        }
    }

    /**
     * set a to 1 if b is greater than c; set it to 0 otherwise
     */
    fn gt(&mut self) {
        let dest:u16 = self.peek_inc();
        let b:u16 = self.peek_inc();
        let c:u16 = self.peek_inc();

        if b > c {
            self.poke(dest, 1);
        } else {
            self.poke(dest, 0);
        }
    }

    /**
     * jump to a
     */
    fn jmp(&mut self) {
        self.pc = self.peek_inc();
    }

    /**
     * if a is nonzero jump to b
     */
    fn jt(&mut self) {
        if self.peek_inc() != 0 {
            self.pc = self.peek_inc();
        }
    }

    /**
     * if a is 0 jump to b
     */
    fn jf(&mut self) {
        if self.peek_inc() == 0 {
            self.pc = self.peek_inc();
        }
    }

    /**
     * Assign into a the sum of immediate values b and c (modulo 0x8000)
     */
    fn add(&mut self) {
        let dest:u16 = self.peek_inc();
        let mut sum:u16 = self.peek_inc();

        // a value between TOM and TOM + NUM_REG inclusive refers to a register location instead
        if sum >= TOM as u16 {
            sum = self.peek(sum);
        }

        sum += self.peek_inc();
        sum %= TOM as u16;

        self.poke(dest, sum);
    }

    /**
     * store into a the product of b and c (modulo 32768)
     */
    fn mult(&mut self) {
        let dest:u16 = self.peek_inc();
        let mut product:u16 = self.peek_inc();

        // a value between TOM and TOM + NUM_REG inclusive refers to a register location instead
        if product >= TOM as u16 {
            product = self.peek(product);
        }

        product *= self.peek_inc();
        product %= TOM as u16;

        self.poke(dest, product)
    }

    /**
     * Writes the character represented by immediate ASCII code a to the terminal
     */
    fn out(&mut self) {
        let mut val:u16 = self.peek_inc();

        if val >= TOM as u16 {
            val = self.peek(val);
        }

        // ASCII output
        set_bit(&mut self.status, OUT_BIT);
        print!("{}", (val as u8) as char);
        io::stdout().flush().unwrap();
        clear_bit(&mut self.status, OUT_BIT);
    }

    /**
     * store into a the remainder of b/c
     */
    fn modulo(&mut self) {
        let dest:u16 = self.peek_inc();
        let b:u16 = self.peek_inc();
        let c:u16 = self.peek_inc();
        self.poke(dest, b%c);
    }

    /**
     * store into a the bitwise and of b and c
     */
    fn and(&mut self) {
        let dest:u16 = self.peek_inc();
        let b:u16 = self.peek_inc();
        let c:u16 = self.peek_inc();
        let value:u16 = (b&c) % TOM as u16;
        self.poke(dest, value);
    }

    /**
     * store into a the bitwise or of b and c
     */
    fn or(&mut self) {
        let dest:u16 = self.peek_inc();
        let b:u16 = self.peek_inc();
        let c:u16 = self.peek_inc();
        let value:u16 = (b|c) % TOM as u16;
        self.poke(dest, value);
    }

    /**
     * store into a the bitwise inverse of b
     */
    fn not(&mut self) {
        let dest:u16 = self.peek_inc();
        let b:u16 = self.peek_inc();
        let value:u16 = (!b) % TOM as u16;
        self.poke(dest, value);
    }


    /**
     * No operation is performed
     */
    fn nop(&self) {
    }
}

// see tests.rs
fn main() {
}

