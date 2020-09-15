
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
    fn test_add() {
        let mut prog:[u16; 7] = [ 0x0900, 0x0080, 0x0180, 0x0400, 0x1300, 0x0080, 0x0000 ];
        //                       add       a     (<b> +   4)     out     <a>         HLT
        let mut m0 = Machine::new();
        for n in 0..6 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x8000), 4);
    }

/*    #[test]
    fn test_add_2() {
        let mut prog:[u16; 7] = [ 0x0900, 0xFF79, 0x0800, 0x0400, 0x1300, 0x0080, 0x0000 ];
        //                           add  0x79FF   (8 +   4)        out   0x8000   HLT
        rmem 0xFF79
        let mut m0 = Machine::new();
        for n in 0..6 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x79FF), 12);
    }
 */

    #[test]
    fn test_example_program_2() {
        let prog:[u16; 4] = [ 0x1300, 0x0300, 0x0000, 0x4100 ];
        // OUT <0x0003> HLT 'A'
        let mut m0 = Machine::new();
        for n in 0..3 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        // TODO: assert that stdout == 'A' somehow
    }

    #[test]
    fn test_set() {
        let prog:[u16; 4] = [ 0x0100, 0x0480, 0xFF00, 0x0000 ];
        // SET e 0x00FF HLT
        let mut m0 = Machine::new();
        for n in 0..4 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x8004), 0x00FF);
        assert_eq!(m0.registers[4], 0xFF00);
    }

    #[test]
    fn test_push() {
        let prog:[u16; 5] = [ 0x0200, 0x00AA, 0x0200, 0xFF00, 0x0000 ];
        //                      PUSH  0xAA00    PUSH  0x00FF    HALT
        let mut m0 = Machine::new();
        for n in 0..4 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.stack[0], 0xAA00);
        assert_eq!(m0.stack[1], 0x00FF);
    }

    #[test]
    fn test_push_pop() {
        let prog:[u16; 9] = [ 0x0200, 0x00AA, 0x0200, 0xFF00, 0x0300, 0x0080, 0x0300, 0x0001, 0x0000 ];
        //                      PUSH  0xAA00    PUSH  0x00FF     POP  0x8000     POP  0x0100    HALT
        let mut m0 = Machine::new();
        for n in 0..8 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x8000), 0x00FF);
        assert_eq!(m0.peek(0x0100), 0xAA00);
    }

    #[test]
    fn test_eq() {
        let prog:[u16; 12] = [ 0x0100, 0x0080, 0xFFFF, 0x0400, 0x0001, 0xAAAA, 0xAAAA, 0x0400, 0x0080, 0xAAAA, 0xAAAB, 0x0000 ];
        //                       SET       A   0xFFFF      EQ  0x0100  0xAAAA  0xAAAA      EQ       A  0xAAAA  0xABAA,   HALT
        let mut m0 = Machine::new();
        for n in 0..11 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x0100), 0x0001);
        assert_eq!(m0.peek(0x8000), 0x0000);
    }

    #[test]
    fn test_gt() {
        let prog:[u16; 12] = [ 0x0100, 0x0080, 0xFFFF, 0x0500, 0x0001, 0xAAAA, 0xAAAA, 0x0500, 0x0080, 0xAAAA, 0xAAA9, 0x0000 ];
        //                       SET       A   0xFFFF      LT  0x0100  0xAAAA  0xAAAA      LT       A  0xAAAA  0xA9AA,   HALT
        let mut m0 = Machine::new();
        for n in 0..11 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x0100), 0x0000);
        assert_eq!(m0.peek(0x8000), 0x0001);
    }

    #[test]
    fn test_jmp() {
        let prog:[u16; 7] = [ 0x0600, 0x0300, 0x0000, 0x0100, 0x0080, 0xFFFF, 0x0000 ];
        //                       JMP  0x0003    HALT     SET       A  0xFFFF    HALT
        let mut m0 = Machine::new();
        for n in 0..6 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x8000), 0xFFFF);
    }

    #[test]
    fn test_jt() {
        let prog:[u16; 7] = [ 0x0700, 0x0100, 0x0600, 0x0100, 0x0080, 0xFFFF, 0x0000 ];
        //                        JT  0x0001  0x0006     SET       a  0xFFFF    HALT
        let mut m0 = Machine::new();
        for n in 0..6 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x8000), 0x0000);
    }

    #[test]
    fn test_jf() {
        let prog:[u16; 7] = [ 0x0800, 0x0000, 0x0600, 0x0100, 0x0080, 0xFFFF, 0x0000 ];
        //                        JF  0x0000  0x0006     SET       a  0xFFFF    HALT
        let mut m0 = Machine::new();
        for n in 0..6 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x8000), 0x0000);
    }

    #[test]
    fn test_mult() {
        let mut prog:[u16; 8] = [ 0x0100, 0x0080, 0xFF00, 0x0A00, 0x0180, 0x0080, 0x0400, 0x0000 ];
        //                           SET       a  0x00FF    MULT       b       a  0x0004    HALT
        let mut m0 = Machine::new();
        for n in 0..7 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x8001), 0x00FF * 4);
    }

    #[test]
    fn test_mod() {
        let mut prog:[u16; 5] = [ 0x0B00, 0x0080, 0xFF00, 0x0A00, 0x0000 ];
        //                           MOD       a  0x00FF  0x000A    HALT

        let mut m0 = Machine::new();
        for n in 0..4 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x8000), 5);
    }

    #[test]
    fn test_and() {
        let mut prog:[u16; 5] = [ 0x0C00, 0x0080, 0xAA00, 0xDEDE, 0x0000 ];
        //                           AND       a  0x00AA  0xDEDE    HALT
        let mut m0 = Machine::new();
        for n in 0..4 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x8000), 0x00AA & (0xDEDE % TOM as u16));
    }

    #[test]
    fn test_or() {
        let mut prog:[u16; 5] = [ 0x0D00, 0x0080, 0xAA00, 0xDE00, 0x0000 ];
        //                            OR       a  0x00AA  0x00DE    HALT
        let mut m0 = Machine::new();
        for n in 0..4 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x8000), 254);
    }

    #[test]
    fn test_not() {
        let mut prog:[u16; 4] = [ 0x0E00, 0x0080, 0xAA00, 0x0000 ];
        //                           NOT       a  0x00AA    HALT
        let mut m0 = Machine::new();
        for n in 0..3 {
            m0.mem[n] = prog[n];
        }
        m0.run();
        assert_eq!(m0.peek(0x8000), (0xFF00 | 0x0055) % TOM as u16);
    }
}
