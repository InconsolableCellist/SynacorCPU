pub const TOM:usize = 0x8000; // Top Of Memory, exclusive (mem: 0x0000-0x7FFF inclusive)
pub const NUM_REG:usize = 8;

// status reg
// x = undefined
// x x IN OUT    MEMW MEMR M1 HLT?
pub const IN_BIT:u16 = 8;      // IO event
pub const OUT_BIT:u16 = 4;      // IO event
pub const MEMW_BIT:u16 = 3;     // mem read
pub const MEMR_BIT:u16 = 2;     // mem read
pub const M1_BIT:u16 = 1;       // M1 cycle
pub const HALT_BIT:u16 = 0;

pub const MAX_RECENTMEMACCESS_SIZE:u8 = 255; // prevents recentMemAccess from growing past this size
pub const RECENTMEMACCESS_READ_BIT:u8 = 1;
pub const RECENTMEMACCESS_WRITE_BIT:u8 = 2;
