use bytes::{Buf, Bytes};

pub trait BufferExtras: Buf {
    fn get_u8_array(&mut self, length: usize) -> Vec<u8> {
        self.get_generic_array(length, |buf| buf.get_u8())
    }

    fn get_u16_array(&mut self) -> Vec<u16> {
        self.get_generic_u16_array(|buf| buf.get_u16())
    }

    fn get_generic_array<T, F>(&mut self, length: usize, element_reader: F) -> Vec<T> where F : Fn(&mut Self) -> T {
        let mut result = Vec::with_capacity(length);
        for _ in 0..length {
            result.push(element_reader(self));
        }
        result
    }

    fn get_generic_u16_array<T, F>(&mut self, element_reader: F) -> Vec<T> where F : Fn(&mut Self) -> T {
        let length = self.get_u16() as usize;
        self.get_generic_array(length, element_reader)
    }
}

impl BufferExtras for Bytes {
}
