use std::mem::MaybeUninit;

pub trait Buffer {
    fn write_info(&mut self) -> (*mut u8, usize);

    /// # Safety
    ///
    /// The returned slice of bytes must have been previously
    /// initialized by calling [`Buffer::write_info`] and writing
    /// `amt` bytes to the returned pointer.
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
        (self[0].as_mut_ptr(), self.len())
    }

    unsafe fn write_data(&mut self, amt: usize) -> &mut [u8] {
        // Copied from std::mem::MaybeUninit::slice_assume_init_mut, which is currently unstable
        &mut *(&mut self[..amt] as *mut Self as *mut [u8])
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
