use crate::machine::Machine;

pub fn disassemble(m0:&mut Machine) {
    println!("disassemble");
}

pub fn save_state(m0:&mut Machine) {
    println!("saving state");
}

pub fn load_state(m0:&mut Machine) {
    println!("loading state");
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
