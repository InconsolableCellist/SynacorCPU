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

const TOM:usize = 0x8000; // Top Of Memory, exclusive (mem: 0x0000-0x7FFF inclusive)
const NUM_REG:usize = 8;
const DEBUG:bool = false;


// status reg
// x = undefined
// x x IN OUT    MEMW MEMR M1 HLT?
const IN_BIT:u16 = 8;      // IO event
const OUT_BIT:u16 = 4;      // IO event
const MEMW_BIT:u16 = 3;     // mem read
const MEMR_BIT:u16 = 2;     // mem read
const M1_BIT:u16 = 1;       // M1 cycle
const HALT_BIT:u16 = 0;

fn set_bit(data:&mut u16, bit_position:u16) {
    *data |= 1 << bit_position;
}

fn clear_bit(data:&mut u16, bit_position:u16) {
    *data &= 0xFFFF ^ (1 << bit_position);
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

pub struct Machine {
    mem:[u16; TOM],
    stack:Vec<u16>,
    registers:[u16; NUM_REG],
    pc:u16,
    status:u16,
    executed:u32,
    recentMemAccess:Vec<(u16, u8)>,  // contains: (memory cell that was read or written to, type of access). To be consumed and pruned by a visualization
}
const MAX_RECENTMEMACCESS_SIZE:u8 = 255; // prevents recentMemAccess from growing past this size
const RECENTMEMACCESS_READ_BIT:u8 = 1;
const RECENTMEMACCESS_WRITE_BIT:u8 = 2;

impl Index<u16> for Machine {
    type Output = u16;
    fn index(&self, addr: u16) -> &u16 {
        if addr < (TOM as u16) {
            &self.mem[addr as usize]
        } else {
            self.dump();
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
            executed: 0,
            recentMemAccess: Vec::new(),
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
        //println!("pc: {:#X}", self.pc);
        let val:u16 = self.mem[self.pc as usize];

        self.pc += 1;

        if self.recentMemAccess.len() < MAX_RECENTMEMACCESS_SIZE as usize {
            self.recentMemAccess.push((self.pc, RECENTMEMACCESS_READ_BIT));
        }
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
        } else if dest_addr < (TOM+NUM_REG) as u16 {
            val = self.registers[(dest_addr % (TOM as u16)) as usize];
        } else {
            panic!(MemoryInvalid);
        }

        if self.recentMemAccess.len() < MAX_RECENTMEMACCESS_SIZE as usize {
            self.recentMemAccess.push((dest_addr, RECENTMEMACCESS_READ_BIT));
        }

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

        if self.recentMemAccess.len() < MAX_RECENTMEMACCESS_SIZE as usize {
            self.recentMemAccess.push((dest_addr, RECENTMEMACCESS_WRITE_BIT));
        }
    }

    fn reset_status(&mut self) {
        clear_bit(&mut self.status, M1_BIT);
        clear_bit(&mut self.status, MEMR_BIT);
        clear_bit(&mut self.status, MEMW_BIT);
        clear_bit(&mut self.status, IN_BIT);
        clear_bit(&mut self.status, OUT_BIT);
        clear_bit(&mut self.status, HALT_BIT);
    }

    /**
     * Performs the M1 operation to fetch the opcode from mem[pc], swaps its endian-ness
     * (from LE to BE) and then executes. The fetch increments `pc`
     */
    pub fn fetch_and_execute(&mut self) {
        if !self.is_halted() {
            self.reset_status();
            set_bit(&mut self.status, M1_BIT);
            let instruction:u16 = self.peek_inc();
            //clear_bit(&mut self.status, M1_BIT);

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
        if DEBUG { println!(" opcode: {:#X} pc: {:#X} (offset {:#X}) step: {} ", instruction, self.pc, self.pc * 2, self.executed); }
        io::stdout().flush().unwrap();
        self.executed += 1;
        match instruction {
            0x0000 => self.halt(),      // `halt`
            0x0001 => self.set(),
            0x0002 => self.push(),
            0x0003 => self.pop(),
            0x0004 => self.eq(),  // a to 1 if b =0 c, 0 otherwise
            0x0005 => self.gt(),
            0x0006 => self.jmp(),
            0x0007 => self.jt(), // jmp if a != 0
            0x0008 => self.jf(), // jmp if a == 0
            0x0009 => self.add(),
            0x000A => self.mult(),
            0x000B => self.modulo(),
            0x000C => self.and(),
            0x000D => self.or(),
            0x000E => self.not(),
            0x000F => self.rmem(),
            0x0010 => self.wmem(),
            0x0011 => self.call(),
            0x0012 => self.ret(),
            0x0013 => self.out(),
            0x0014 => self.read_in(),
            0x0015 => self.nop(),       // `noop` 0d2
            _ => self.unknown(instruction),
        }
    }

    fn unknown(&mut self, instruction:u16) {
        self.dump();
        println!("\n**** unknown opcode ****\n(big-endian)");
        println!("unknown instruction: {:#X}", instruction);
        panic!(UnknownOpcode)
    }

    fn dump(&self) {
        println!("\n**** dump ****\n(big-endian)");
        println!("status: {:#b}", self.status);
        println!("pc: {:#X}\nmem[pc]: {:#X}\n", self.pc, swap_endian(self.mem[self.pc as usize]));
        println!("(file offset {:#X})", self.pc * 2);
        /*
        for x in (0..self.mem.len()) {
            if x % 8 == 0 {
                println!("");
                print!("0000:{:04X}", x);
            }
            print!(" {:04X}", self.mem[x]);
        }
        */
        let mut val:u8;
        for x in (0..self.mem.len()).step_by(8) {
            print!("0000:{:04X}", x);
            for y in x..x+8 {
                print!(" {:04X}", self.mem[y]);
            }
            print!(" ");

            for y in x..x+8 {
                val = (self.mem[y] >> 8) as u8;
                if val >= 0x20 && val <= 0x7E {
                    print!("{}", val as char);
                } else {
                    print!(".");
                }

                val = (self.mem[y] | 0x00FF) as u8;
                if val >= 0x20 && val <= 0x7E {
                    print!("{}", val as char);
                } else {
                    print!(".");
                }
            }
            println!("");
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
        let mut val:u16 = self.peek_inc();

        if val >= TOM as u16 {
            val = self.peek(val);
        }

        self.poke(dest, val);
    }

    /**
     * Pushes immediate value a onto the stack
     */
    fn push(&mut self) {
        let mut val:u16 = self.peek_inc();
        if val >= TOM as u16 {
            val = self.peek(val);
        }
        self.stack.push(val);
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
        let mut b:u16 = self.peek_inc();
        let mut c:u16 = self.peek_inc();

        // a value between TOM and TOM + NUM_REG inclusive refers to a register location instead
        if b >= TOM as u16 {
            b = self.peek(b);
        }
        if c >= TOM as u16 {
            c = self.peek(c);
        }

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
        let mut b:u16 = self.peek_inc();
        let mut c:u16 = self.peek_inc();

        // a value between TOM and TOM + NUM_REG inclusive refers to a register location instead
        if b >= TOM as u16 {
            b = self.peek(b);
        }
        if c >= TOM as u16 {
            c = self.peek(c);
        }

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
        if DEBUG { print!(" (jmp {:#X} (byte offset in file: {:#X})) ", self.pc, self.pc * 2); }
    }

    /**
     * if a is nonzero jump to b
     */
    fn jt(&mut self) {
        let mut val = self.peek_inc();

        // a value between TOM and TOM + NUM_REG inclusive refers to a register location instead
        if val >= TOM as u16 {
            val = self.peek(val);
        }

        let dest = self.peek_inc();
        if val != 0 {
            self.pc = dest;
            if DEBUG { print!(" (jt {:#X} (byte offset in file: {:#X})) ", self.pc, self.pc * 2); }
        }
    }

    /**
     * if a is 0 jump to b
     */
    fn jf(&mut self) {
        let mut val = self.peek_inc();
        let dest = self.peek_inc();

        // a value between TOM and TOM + NUM_REG inclusive refers to a register location instead
        if val >= TOM as u16 {
            val = self.peek(val);
        }

        if val == 0 {
            self.pc = dest;
            if DEBUG { print!(" (jf {:#X} (byte offset in file: {:#X})) ", self.pc, self.pc * 2); }
        }
    }

    /**
     * Assign into a the sum of immediate values b and c (modulo 0x8000)
     */
    fn add(&mut self) {
        let dest:u16 = self.peek_inc();
        let mut a:u16 = self.peek_inc();
        let mut b:u16 = self.peek_inc();

        // a value between TOM and TOM + NUM_REG inclusive refers to a register location instead
        if a >= TOM as u16 {
            a = self.peek(a);
        }
        if b >= TOM as u16 {
            b = self.peek(b);
        }

        //let sum:u16 = a.wrapping_add(b) % TOM as u16;
        let sum:u16 = a.wrapping_add(b) % TOM as u16;

        self.poke(dest, sum);
    }

    /**
     * store into a the product of b and c (modulo 32768)
     */
    fn mult(&mut self) {
        let dest:u16 = self.peek_inc();
        let mut b:u16 = self.peek_inc();
        let mut c:u16 = self.peek_inc();

        // a value between TOM and TOM + NUM_REG inclusive refers to a register location instead
        if b >= TOM as u16 {
            b = self.peek(b);
        }
        if c >= TOM as u16 {
            c = self.peek(c);
        }

        if DEBUG { println!("b: {:#X} c: {:#X}", b, c); }
        b = b.wrapping_mul(c) % TOM as u16;

        self.poke(dest, b)
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
        //clear_bit(&mut self.status, OUT_BIT);
    }

    /**
     * store into a the remainder of b/c
     */
    fn modulo(&mut self) {
        let dest:u16 = self.peek_inc();
        let mut b:u16 = self.peek_inc();
        let mut c:u16 = self.peek_inc();

        if DEBUG { println!("dest: {}, b: {}, c: {}", dest, b, c); }

        // a value between TOM and TOM + NUM_REG inclusive refers to a register location instead
        if b >= TOM as u16 {
            b = self.peek(b);
        }
        if c >= TOM as u16 {
            c = self.peek(c);
        }

        if DEBUG { println!("dest: {}, b: {}, c: {}", dest, b, c); }

        self.poke(dest, b%c);
    }

    /**
     * store into a the bitwise and of b and c
     */
    fn and(&mut self) {
        let dest:u16 = self.peek_inc();
        let mut b:u16 = self.peek_inc();
        let mut c:u16 = self.peek_inc();

        if b >= TOM as u16 {
            b = self.peek(b);
        }
        if c >= TOM as u16 {
            c = self.peek(c);
        }

        let value:u16 = (b&c) % TOM as u16;
        self.poke(dest, value);
    }

    /**
     * store into a the bitwise or of b and c
     */
    fn or(&mut self) {
        let dest:u16 = self.peek_inc();
        let mut b:u16 = self.peek_inc();
        let mut c:u16 = self.peek_inc();

        if b >= TOM as u16 {
            b = self.peek(b);
        }
        if c >= TOM as u16 {
            c = self.peek(c);
        }

        let value:u16 = (b|c) % TOM as u16;
        self.poke(dest, value);
    }

    /**
     * store into a the bitwise inverse of b
     */
    fn not(&mut self) {
        let dest:u16 = self.peek_inc();
        let mut b:u16 = self.peek_inc();
        if b >= TOM as u16 {
            b = self.peek(b);
        }
        let value:u16 = (!b) % TOM as u16;
        self.poke(dest, value);
    }

    /**
     * read memory at address <b> and write it to address in <a>
     */
    fn rmem(&mut self) {
        let dest:u16 = self.peek_inc();
        let mut source:u16 = self.peek_inc();

        if source >= TOM as u16 {
            source = self.peek(source);
        }
        let mut value:u16 = self.peek(source);

        if value >= TOM as u16 {
            value = self.peek(value);
        }
        if DEBUG { println!("storing into {:#X} the value contained in {:#X}, which is {:#X}", dest, source, value); }

        self.poke(dest, value);
    }

    /**
     * write value contained in <b> into memory at address <a>
     */
    fn wmem(&mut self) {
        let mut dest:u16 = self.peek_inc();
        let mut value:u16 = self.peek_inc();

        if dest >= TOM as u16 {
            dest = self.peek(dest);
        }
        if value >= TOM as u16 {
            value = self.peek(value);
        }
        if DEBUG { println!("writing mem location {:#X} with the value {:#X}", dest, value); }

        self.poke(dest, value);
    }

    /**
     * write the address of the next instruction to the stack and jump to a
     */
    fn call(&mut self) {
        let mut dest:u16 = self.peek_inc();
        self.stack.push(self.pc);
        if dest >= TOM as u16 {
            dest = self.peek(dest);
        }
        self.pc = dest;
    }

    /**
     * remove the top element from the stack and jump to it; empty stack = halt
     */
    fn ret(&mut self) {
        let value:u16 = match self.stack.pop() {
            Some(p) => p,
            None => panic!(EmptyStack)
        };
        self.pc = value;
    }

    /**
     * Read a character from the terminal and write its ascii code to a
     * It can be assumed that once input starts, it will continue
     * until a newline is encountered.
     * This means that you can safely read whole lines from the keyboard
     * and trust that they will be fully read
     */
    fn read_in(&mut self) {
        let dest:u16 = self.peek_inc();
        let mut input = String::new();
        set_bit(&mut self.status, IN_BIT);
        std::io::stdin().read_line(&mut input).ok().expect(&FailedToReadLine.to_string());
        //clear_bit(&mut self.status, IN_BIT);

        let bytes = input.bytes().nth(0).expect("no byte read");
        self.poke(dest, bytes as u16);
    }

    /**
     * No operation is performed
     */
    fn nop(&self) {
    }
}

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
        m0.mem[x as usize] = val;
        x+=1;
    }

    frontpanelRun(&mut m0);

    // m0.run();

    Ok(())
}

