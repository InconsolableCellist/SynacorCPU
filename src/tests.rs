
#[cfg(test)]
mod tests {
    use crate::{Machine, TOM, NUM_REG};
    use std::num::Wrapping;

    #[test]
    fn test_mem_rw() {
        let mut m0 = Machine::new();

        assert_eq!((m0[(TOM - 1) as u16]), 0); // last u16 in memory
        assert_eq!((m0[(0) as u16]), 0);

        m0.mem[TOM-1] = 0x0F0F;
        m0.mem[0] = 0xAA00;

        assert_eq!((m0[(TOM - 1) as u16]), 0x0F0F);
        assert_eq!((m0[(0) as u16]), 0xAA00);
    }

    #[test]
    fn test_registers_rw() {
        let mut m0 = Machine::new();
        assert_eq!(m0.registers[0], 0);
        assert_eq!(m0.registers[NUM_REG-1], 0);

        m0.registers[0] = 0x0F0F;
        m0.registers[7] = 0xAA00;

        assert_eq!(m0.registers[0], 0x0F0F);
        assert_eq!(m0.registers[NUM_REG-1], 0xAA00);

        /*
        m0.registers[0] = 0xFFFF;
        m0.registers[0] = m0.registers[0] + 1;

        assert_eq!(m0.registers[0], 0x0000);
        // carry bit
     */
    }

    #[test]
    #[should_panic]
    fn test_mem_read_invalid() {
        let mut m0 = Machine::new();
        assert_eq!((m0[(TOM) as u16]), 0);
    }

    #[test]
    #[should_panic]
    fn test_mem_read_invalid_1() {
        let mut m0 = Machine::new();
        assert_eq!((m0[(TOM+1) as u16]), 0);
    }

    #[test]
    fn test_halt() {
        let mut m0 = Machine::new();
        assert_eq!(m0.is_halted(), false);
        m0.halt();
        assert_eq!(m0.is_halted(), true);
    }

    #[test]
    fn test_halt_program() {
        let mut m0 = Machine::new();
        m0.mem[0] = 0x00;
        assert_eq!(m0.is_halted(), false);
        m0.fetch_and_execute();
        assert_eq!(m0.is_halted(), true);
    }

    #[test]
    #[should_panic]
    fn test_halt_program_invalid() {
        let mut m0 = Machine::new();
        m0.mem[0] = 0x00FF; // unknown opcode
        m0.mem[1] = 0xFF00;
        m0.mem[2] = 0x0000;
        assert_eq!(m0.is_halted(), false);
        m0.fetch_and_execute();
        assert_eq!(m0.is_halted(), false);
        m0.fetch_and_execute();
        assert_eq!(m0.is_halted(), false);
        m0.fetch_and_execute();
        assert_eq!(m0.is_halted(), true);
    }

    #[test]
    fn test_example_program_1() {
        let prog:[u16; 6] = [ 0x0900, 0x0080, 0x0180, 0x0400, 0x1300, 0x0080 ];
        //                       add     <a>     (<b> +   4)     out     <a>
        let mut m0 = Machine::new();
        for n in 0..6 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x8000), 4);
    }

    #[test]
    fn test_example_program_2() {
        let prog:[u16; 4] = [ 0x1300, 0x0300, 0x0000, 0x4100 ];
        // OUT <0x0003> HLT 'A'
        let mut m0 = Machine::new();
        for n in 0..4 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        // TODO: assert that stdout == 'A' somehow
    }

    #[test]
    fn test_set() {
        let prog:[u16; 5] = [ 0x0100, 0x0480, 0x0400, 0x0000, 0xFF00 ];
        // SET e 0x00FF HLT
        let mut m0 = Machine::new();
        for n in 0..5 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x8004), 0x00FF);
        assert_eq!(m0.registers[4], 0xFF00);
    }

    #[test]
    fn test_push() {
        let prog:[u16; 7] = [ 0x0200, 0x0500, 0x0200, 0x0600, 0x0000, 0xAA00, 0x00FF ];
        // PUSH <0x0005> PUSH <0x0006> HLT
        //      (0x00AA)      (0xFF00)
        let mut m0 = Machine::new();
        for n in 0..5 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.stack[0], 0xAA00);
        assert_eq!(m0.stack[1], 0x00FF);
    }

}
