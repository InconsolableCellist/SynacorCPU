pub fn set_bit(data:&mut u16, bit_position:u16) {
    *data |= 1 << bit_position;
}

pub fn clear_bit(data:&mut u16, bit_position:u16) {
    *data &= 0xFFFF ^ (1 << bit_position);
}

pub fn get_bit(data:&u16, bit_position:u16) -> bool {
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
pub fn swap_endian(ushort:u16) -> u16 {
    (ushort << 8) | (ushort >> 8)
}
