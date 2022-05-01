use crate::Buffer;

use libaec_sys::szlib::*;
use libc::{c_int, c_void, size_t};

bitflags::bitflags! {
    pub struct Options: u32 {
        const ALLOW_K13 = SZ_ALLOW_K13_OPTION_MASK as u32;
        const CHIP = SZ_CHIP_OPTION_MASK as u32;
        const EC = SZ_EC_OPTION_MASK as u32;
        const LSB = SZ_LSB_OPTION_MASK as u32;
        const MSB = SZ_MSB_OPTION_MASK as u32;
        const NN = SZ_NN_OPTION_MASK as u32;
        const RAW = SZ_RAW_OPTION_MASK as u32;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Error {
    OutputBufferFull,
    Parameter,
    Memory,
}

impl Error {
    fn from_int(v: c_int) -> Result<(), Self> {
        match v {
            SZ_OK => Ok(()),
            SZ_OUTBUFF_FULL => Err(Self::OutputBufferFull),
            SZ_PARAM_ERROR => Err(Self::Parameter),
            SZ_MEM_ERROR => Err(Self::Memory),
            // this is a lie, but it should also never happen
            // and I'd rather *this* than panic
            _ => Err(Self::Parameter),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::OutputBufferFull => write!(f, "output buffer is full"),
            Self::Parameter => write!(f, "bad parameters"),
            Self::Memory => write!(f, "out of memory"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for std::io::Error {
    fn from(err: Error) -> Self {
        use std::io::ErrorKind;
        let kind = match err {
            Error::OutputBufferFull => ErrorKind::InvalidInput,
            Error::Parameter => ErrorKind::InvalidInput,
            Error::Memory => ErrorKind::Other,
        };
        std::io::Error::new(kind, err)
    }
}

#[derive(Clone, Debug)]
pub struct Parameters(SZ_com_t);

impl Parameters {
    pub fn new(
        options: Options,
        bits_per_pixel: usize,
        pixels_per_block: usize,
        pixels_per_scanline: usize,
    ) -> Self {
        Self(SZ_com_t {
            options_mask: options.bits() as c_int,
            bits_per_pixel: bits_per_pixel as c_int,
            pixels_per_block: pixels_per_block as c_int,
            pixels_per_scanline: pixels_per_scanline as c_int,
        })
    }

    pub fn compress<'a, B>(&mut self, source: &[u8], dest: &'a mut B) -> Result<&'a mut [u8], Error>
    where
        B: Buffer + ?Sized,
    {
        let (destptr, destlen) = dest.write_info();
        let mut destlen = destlen as size_t;
        let destptr = destptr as *mut c_void;
        let sourcelen = source.len() as size_t;
        let sourceptr = source.as_ptr() as *const c_void;
        unsafe {
            let result =
                SZ_BufftoBuffCompress(destptr, &mut destlen, sourceptr, sourcelen, &mut self.0);
            Error::from_int(result).map(move |_| dest.write_data(destlen as usize))
        }
    }

    pub fn decompress<'a, B>(
        &mut self,
        source: &[u8],
        dest: &'a mut B,
    ) -> Result<&'a mut [u8], Error>
    where
        B: Buffer + ?Sized,
    {
        let (destptr, destlen) = dest.write_info();
        let mut destlen = destlen as size_t;
        let destptr = destptr as *mut c_void;
        let sourcelen = source.len() as size_t;
        let sourceptr = source.as_ptr() as *const c_void;
        unsafe {
            let result =
                SZ_BufftoBuffDecompress(destptr, &mut destlen, sourceptr, sourcelen, &mut self.0);
            Error::from_int(result).map(move |_| dest.write_data(destlen as usize))
        }
    }

    pub fn options(&self) -> Options {
        Options::from_bits_truncate(self.0.options_mask as u32)
    }

    pub fn bits_per_pixel(&self) -> usize {
        self.0.bits_per_pixel as usize
    }

    pub fn pixels_per_block(&self) -> usize {
        self.0.pixels_per_block as usize
    }

    pub fn pixels_per_scanline(&self) -> usize {
        self.0.pixels_per_scanline as usize
    }
}

#[cfg(test)]
mod test {
    use super::{Options, Parameters};

    #[test]
    fn round_trip() {
        let pixels_per_block = 16;
        let data = b" This is a fun message for you. ";
        assert_eq!(data.len() % pixels_per_block, 0);

        let mut compressed = Vec::with_capacity(data.len() + 1);
        compressed.push(42);
        let mut decompressed = Vec::with_capacity(data.len() + 1);
        decompressed.push(42);

        let mut c = Parameters::new(
            Options::ALLOW_K13 | Options::MSB | Options::NN,
            8,
            pixels_per_block,
            data.len(),
        );

        assert!(c.compress(data, &mut compressed).is_ok());
        assert_eq!(compressed[0], 42);
        assert!(c.decompress(&compressed[1..], &mut decompressed).is_ok());
        assert_eq!(decompressed[0], 42);
        assert_eq!(&decompressed[1..], data);
    }
}
