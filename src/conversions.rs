use bytes::{Buf, Bytes};

pub fn u16_to_u8_array(value: u16) -> [u8; 2] {
    [(value >> 8) as u8, value as u8]
}

pub fn read_u16_array(mut buf: &Bytes) -> Vec<u16> {
    let count = buf.get_u16();
    let mut array = Vec::with_capacity(count as usize);
    for _ in 0..count {
        array.push(buf.get_u16());
    }
    array
}
