use std::mem::MaybeUninit;

pub trait Buffer {
    fn write_info(&mut self) -> (*mut u8, usize);
    unsafe fn write_data(&mut self, amt: usize) -> &mut [u8];
}

impl Buffer for [u8] {
    fn write_info(&mut self) -> (*mut u8, usize) {
        (self.as_mut_ptr(), self.len())
    }

    unsafe fn write_data(&mut self, amt: usize) -> &mut [u8] {
        &mut self[..amt]
    }
}

impl Buffer for [MaybeUninit<u8>] {
    fn write_info(&mut self) -> (*mut u8, usize) {
        (self.as_mut_ptr() as *mut u8, self.len())
    }

    unsafe fn write_data(&mut self, amt: usize) -> &mut [u8] {
        std::mem::transmute::<_, &mut [u8]>(&mut self[..amt])
    }
}

impl Buffer for Vec<u8> {
    fn write_info(&mut self) -> (*mut u8, usize) {
        let l = self.len();
        (self[l..].as_mut_ptr(), self.capacity() - l)
    }

    unsafe fn write_data(&mut self, amt: usize) -> &mut [u8] {
        let l = self.len();
        self.set_len(l + amt);
        &mut self[l..]
    }
}
