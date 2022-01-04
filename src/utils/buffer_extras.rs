use bytes::{Buf, Bytes};

pub fn read_u16_array(buf: &mut Bytes) -> Vec<u16> {
    let count = buf.get_u16();
    let mut array = Vec::with_capacity(count as usize);
    for _ in 0..count {
        array.push(buf.get_u16());
    }
    array
}
