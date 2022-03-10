use crate::machine::Machine;
use serde::{Serialize, Deserialize};
use std::{fs, io};
use crate::utils::swap_endian;
use std::io::{Read, Write};
use std::str::Split;
use crate::constants::TOM;

pub fn write_memory(m0:&mut Machine) {
    println!("write memory");

    let mut buffer:String = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    buffer.pop(); // remove \n
    let tokens:Vec<&str> = buffer.split(" ").collect();
    if tokens.len() < 3 {
        println!("Usage: w NNNN v
        NNNN - memory location in HEX
        v - value in HEX");
    } else {
        io::stdout().flush().unwrap();

        let loc:u16 = i64::from_str_radix(tokens[1], 16).unwrap() as u16;
        let val:u16 = i64::from_str_radix(tokens[2], 16).unwrap() as u16;
        if (loc <= TOM as u16) {
            m0.mem[loc as usize] = swap_endian(val);
        } else {
            println!("Invalid params");
        }
    }

}


pub fn disassemble(m0:&mut Machine) {
    println!("disassemble");

    let mut buffer:String = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    buffer.pop(); // remove \n
    let tokens:Vec<&str> = buffer.split(" ").collect();
    if tokens.len() < 3 {
        println!("Usage: d SSSS EEEE
        SSSS - starting address in HEX
        EEEE - ending address in HEX");
    } else {
        io::stdout().flush().unwrap();

        let start:u16 = i64::from_str_radix(tokens[1], 16).unwrap() as u16;
        let end:u16 = i64::from_str_radix(tokens[2], 16).unwrap() as u16;
        if (start <= end) && (end <= TOM as u16) {
            disassemble_range(m0, start, end);
        } else {
            println!("Invalid params");
        }
    }
}

pub fn disassemble_range(m0:&Machine, start:u16, end:u16) {
    let mut addr:u16 = start;
    while addr <= end {
        let opcode:u16 = swap_endian(m0.mem[addr as usize]);
        let a1:u16 = swap_endian(m0.mem[(addr+1) as usize]);
        let a2:u16 = swap_endian(m0.mem[(addr+2) as usize]);
        let a3:u16 = swap_endian(m0.mem[(addr+3) as usize]);

        match opcode {
            0x0000 => { println!("{:#06X}:\thalt", addr);                               addr += 2; },
            0x0001 => { println!("{:#06X}:\tset\t\t{:#06X}\t{:#06X}", addr, a1, a2);    addr += 3; },
            0x0002 => { println!("{:#06X}:\tpush\t{:#06X}", addr, a1);                  addr += 2; },
            0x0003 => { println!("{:#06X}:\tpop\t\t{:#06X}", addr, a1);                 addr += 2; },
            0x0004 => { println!("{:#06X}:\teq\t\t{:#06X}\t{:#06X}\t{:#06X}", addr, a1, a2, a3); addr += 4 },
            0x0005 => { println!("{:#06X}:\tgt\t\t{:#06X}\t{:#06X}\t{:#06X}", addr, a1, a2, a3); addr += 4 },
            0x0006 => { println!("{:#06X}:\tjmp\t\t{:#06X}", addr, a1);                 addr += 2 },
            0x0007 => { println!("{:#06X}:\tjt\t\t{:#06X}\t{:#06X}", addr, a1, a2);     addr += 3 },
            0x0008 => { println!("{:#06X}:\tjf\t\t{:#06X}\t{:#06X}", addr, a1, a2);     addr += 3 },
            0x0009 => { println!("{:#06X}:\tadd\t\t{:#06X}\t{:#06X}\t{:#06X}", addr, a1, a2, a3);     addr += 4 },
            0x000A => { println!("{:#06X}:\tmult\t\t{:#06X}\t{:#06X}\t{:#06X}", addr, a1, a2, a3);    addr += 4 },
            0x000B => { println!("{:#06X}:\tmod\t\t{:#06X}\t{:#06X}\t{:#06X}", addr, a1, a2, a3);     addr += 4 },
            0x000C => { println!("{:#06X}:\tand\t\t{:#06X}\t{:#06X}\t{:#06X}", addr, a1, a2, a3);     addr += 4 },
            0x000D => { println!("{:#06X}:\tor\t\t{:#06X}\t{:#06X}\t{:#06X}", addr, a1, a2, a3);      addr += 4 },
            0x000E => { println!("{:#06X}:\tnot\t\t{:#06X}\t{:#06X}", addr, a1, a2);     addr += 3 },
            0x000F => { println!("{:#06X}:\trmem\t{:#06X}\t{:#06X}", addr, a1, a2);     addr += 3 },
            0x0010 => { println!("{:#06X}:\twmem\t{:#06X}\t{:#06X}", addr, a1, a2);     addr += 3 },
            0x0011 => { println!("{:#06X}:\tcall\t{:#06X}", addr, a1);      addr += 2 },
            0x0012 => { println!("{:#06X}:\tret", addr);                    addr += 1 },
            0x0013 => { println!("{:#06X}:\tout\t\t{:#06X}", addr, a1);     addr += 2 },
            0x0014 => { println!("{:#06X}:\tin\t\t{:#06X}", addr, a1);      addr += 2 },
            0x0015 => { println!("{:#06X}:\tnop", addr);                    addr += 1; },
             _ => { println!("{:#06X}:\t??? ({:#06X})", addr, opcode);      addr += 1; },
        }
    }
}

pub fn save_state(m0:&mut Machine) {
    println!("saving state");
    let serialized = serde_json::to_string(&m0).unwrap();
    fs::write("state0.bin", serialized).unwrap();
}

pub fn load_state(m0:&mut Machine) {
    println!("loading state");
    let str:String = fs::read_to_string("state0.bin").unwrap();
    let deserialized:Machine = serde_json::from_str(&str).unwrap();

    for x in 0..deserialized.mem.len() {
        m0.mem[x] = deserialized.mem[x];
    }
    m0.status = deserialized.status;
    m0.recentMemAccess = deserialized.recentMemAccess;
}

pub fn print_regs(m0:&mut Machine) {
    println!("printing registers");
}

pub fn goto_and_run(m0:&mut Machine) {
    println!("goto and run");
}

pub fn examine_memory(m0:&mut Machine) {
    println!("examine memory");
}

pub fn toggle_debug(m0:&mut Machine) {
    print!("toggling debug output ");
    m0.debug ^= true;
    if (m0.debug) {
        println!("on");
    } else {
        println!("off");
    }
}

