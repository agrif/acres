use std::mem::MaybeUninit;

use libaec_sys::*;
use libc::{c_int, c_uint, size_t};

mod buffer;
pub use buffer::Buffer;

mod io;
pub use io::{Reader, Writer};

pub mod sz;

const DEFAULT_BUFFER_SIZE: usize = 8192;

bitflags::bitflags! {
    pub struct Flags: u32 {
        const DATA_SIGNED = AEC_DATA_SIGNED;
        const DATA_3BYTE = AEC_DATA_3BYTE;
        const DATA_MSB = AEC_DATA_MSB;
        const DATA_PREPROCESS = AEC_DATA_PREPROCESS;
        const RESTRICTED = AEC_RESTRICTED;
        const PAD_RSI = AEC_PAD_RSI;
        const NOT_ENFORCE = AEC_NOT_ENFORCE;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Error {
    Configuration,
    Stream,
    Data,
    Memory,
}

impl Error {
    fn from_int(v: c_int) -> Result<(), Self> {
        match v {
            AEC_OK => Ok(()),
            AEC_CONF_ERROR => Err(Self::Configuration),
            AEC_STREAM_ERROR => Err(Self::Stream),
            AEC_DATA_ERROR => Err(Self::Data),
            AEC_MEM_ERROR => Err(Self::Memory),
            // this is a lie, but it should also never happen
            // and I'd rather *this* than panic
            _ => Err(Self::Configuration),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Configuration => write!(f, "bad configuration"),
            Self::Stream => write!(f, "stream closed before all data written"),
            Self::Data => write!(f, "unexpected data"),
            // Memory may also be a poorly-sized output buffer
            Self::Memory => write!(f, "out of memory"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for std::io::Error {
    fn from(err: Error) -> Self {
        use std::io::ErrorKind;
        let kind = match err {
            Error::Configuration => ErrorKind::InvalidInput,
            Error::Stream => ErrorKind::Other,
            Error::Data => ErrorKind::InvalidData,
            Error::Memory => ErrorKind::Other,
        };
        std::io::Error::new(kind, err)
    }
}

#[derive(Debug)]
pub struct Configuration(aec_stream);

impl Clone for Configuration {
    fn clone(&self) -> Self {
        Self(aec_stream { ..self.0 })
    }
}

impl Configuration {
    pub fn new(bits_per_sample: usize, block_size: usize, rsi: usize, flags: Flags) -> Self {
        Self(aec_stream {
            next_in: std::ptr::null(),
            avail_in: 0,
            total_in: 0,
            next_out: std::ptr::null_mut(),
            avail_out: 0,
            total_out: 0,
            bits_per_sample: bits_per_sample as c_uint,
            block_size: block_size as c_uint,
            rsi: rsi as c_uint,
            flags: flags.bits() as c_uint,
            state: std::ptr::null_mut(),
        })
    }

    pub fn encoder(&self) -> Result<Encoder, Error> {
        Encoder::new(aec_stream { ..self.0 })
    }

    pub fn decoder(&self) -> Result<Decoder, Error> {
        Decoder::new(aec_stream { ..self.0 })
    }

    pub fn encode_buffer<'a>(
        &self,
        mut input: &[u8],
        output: &'a mut Vec<u8>,
    ) -> Result<&'a mut [u8], Error> {
        let start = output.len();
        let mut enc = self.encoder()?;
        loop {
            let flush = input.is_empty();
            if flush {
                // give us a bit extra to work with, because we
                // cannot tell when flushing is done
                output.reserve(DEFAULT_BUFFER_SIZE);
            }
            let (rest, _) = enc.encode_vec(input, output, flush)?;
            input = rest;
            if output.len() == output.capacity() {
                output.reserve(DEFAULT_BUFFER_SIZE);
                continue;
            }
            if input.is_empty() && flush {
                break;
            }
        }
        enc.end()?;
        Ok(&mut output[start..])
    }

    pub fn encode_reader<T>(
        &self,
        inner: T,
    ) -> Result<std::io::BufReader<Reader<Encoder, T>>, Error>
    where
        T: std::io::BufRead,
    {
        Ok(Reader::new(self.encoder()?, inner))
    }

    pub fn encode_writer<T>(&self, inner: T) -> Result<Writer<Encoder, T>, Error> {
        Ok(Writer::new(self.encoder()?, inner))
    }

    pub fn decode_buffer<'a>(
        &self,
        mut input: &[u8],
        output: &'a mut Vec<u8>,
    ) -> Result<&'a mut [u8], Error> {
        let start = output.len();
        let mut dec = self.decoder()?;
        loop {
            let (rest, _) = dec.decode_vec(input, output, false)?;
            input = rest;
            if input.is_empty() {
                break;
            }
            if output.len() == output.capacity() {
                output.reserve(DEFAULT_BUFFER_SIZE);
            }
        }
        dec.end()?;
        Ok(&mut output[start..])
    }

    pub fn decode_reader<T>(
        &self,
        inner: T,
    ) -> Result<std::io::BufReader<Reader<Decoder, T>>, Error>
    where
        T: std::io::BufRead,
    {
        Ok(Reader::new(self.decoder()?, inner))
    }

    pub fn decode_writer<T>(&self, inner: T) -> Result<Writer<Decoder, T>, Error> {
        Ok(Writer::new(self.decoder()?, inner))
    }
}

#[derive(Debug)]
pub struct Encoder(aec_stream);

impl Encoder {
    fn new(strm: aec_stream) -> Result<Self, Error> {
        let mut s = Self(strm);
        unsafe { Error::from_int(aec_encode_init(&mut s.0)).map(|_| s) }
    }

    pub fn end(&mut self) -> Result<(), Error> {
        if !self.0.state.is_null() {
            let r = unsafe { aec_encode_end(&mut self.0) };
            self.0.state = std::ptr::null_mut();
            Error::from_int(r)
        } else {
            Ok(())
        }
    }

    pub fn is_ended(&self) -> bool {
        self.0.state.is_null()
    }

    pub fn encode_generic<'i, 'o, B>(
        &mut self,
        input: &'i [u8],
        output: &'o mut B,
        flush: bool,
    ) -> Result<(&'i [u8], &'o mut [u8]), Error>
    where
        B: Buffer + ?Sized,
    {
        if self.0.state.is_null() {
            return Err(Error::Stream);
        }
        self.0.next_in = input.as_ptr();
        self.0.avail_in = input.len() as size_t;
        let (outptr, outlen) = output.write_info();
        self.0.next_out = outptr;
        self.0.avail_out = outlen as size_t;
        let flush = if flush { AEC_FLUSH } else { AEC_NO_FLUSH };
        unsafe {
            let result = aec_encode(&mut self.0, flush);
            Error::from_int(result).map(move |_| {
                (
                    &input[input.len() - self.0.avail_in..],
                    output.write_data(outlen - self.0.avail_out),
                )
            })
        }
    }

    pub fn encode<'i, 'o>(
        &mut self,
        input: &'i [u8],
        output: &'o mut [u8],
        flush: bool,
    ) -> Result<(&'i [u8], &'o mut [u8]), Error> {
        self.encode_generic(input, output, flush)
    }

    pub fn encode_uninit<'i, 'o>(
        &mut self,
        input: &'i [u8],
        output: &'o mut [MaybeUninit<u8>],
        flush: bool,
    ) -> Result<(&'i [u8], &'o mut [u8]), Error> {
        self.encode_generic(input, output, flush)
    }

    pub fn encode_vec<'i, 'o>(
        &mut self,
        input: &'i [u8],
        output: &'o mut Vec<u8>,
        flush: bool,
    ) -> Result<(&'i [u8], &'o mut [u8]), Error> {
        self.encode_generic(input, output, flush)
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        let _ = self.end();
    }
}

#[derive(Debug)]
pub struct Decoder(aec_stream);

impl Decoder {
    fn new(strm: aec_stream) -> Result<Self, Error> {
        let mut s = Self(strm);
        unsafe { Error::from_int(aec_decode_init(&mut s.0)).map(|_| s) }
    }

    pub fn end(&mut self) -> Result<(), Error> {
        if !self.0.state.is_null() {
            let r = unsafe { aec_decode_end(&mut self.0) };
            self.0.state = std::ptr::null_mut();
            Error::from_int(r)
        } else {
            Ok(())
        }
    }

    pub fn is_ended(&self) -> bool {
        self.0.state.is_null()
    }

    pub fn decode_generic<'i, 'o, B>(
        &mut self,
        input: &'i [u8],
        output: &'o mut B,
        flush: bool,
    ) -> Result<(&'i [u8], &'o mut [u8]), Error>
    where
        B: Buffer + ?Sized,
    {
        if self.0.state.is_null() {
            return Err(Error::Stream);
        }
        self.0.next_in = input.as_ptr();
        self.0.avail_in = input.len() as size_t;
        let (outptr, outlen) = output.write_info();
        self.0.next_out = outptr;
        self.0.avail_out = outlen as size_t;
        let flush = if flush { AEC_FLUSH } else { AEC_NO_FLUSH };
        unsafe {
            let result = aec_decode(&mut self.0, flush);
            Error::from_int(result).map(move |_| {
                (
                    &input[input.len() - self.0.avail_in..],
                    output.write_data(outlen - self.0.avail_out),
                )
            })
        }
    }

    pub fn decode<'i, 'o>(
        &mut self,
        input: &'i [u8],
        output: &'o mut [u8],
        flush: bool,
    ) -> Result<(&'i [u8], &'o mut [u8]), Error> {
        self.decode_generic(input, output, flush)
    }

    pub fn decode_uninit<'i, 'o>(
        &mut self,
        input: &'i [u8],
        output: &'o mut [MaybeUninit<u8>],
        flush: bool,
    ) -> Result<(&'i [u8], &'o mut [u8]), Error> {
        self.decode_generic(input, output, flush)
    }

    pub fn decode_vec<'i, 'o>(
        &mut self,
        input: &'i [u8],
        output: &'o mut Vec<u8>,
        flush: bool,
    ) -> Result<(&'i [u8], &'o mut [u8]), Error> {
        self.decode_generic(input, output, flush)
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        let _ = self.end();
    }
}

#[cfg(test)]
mod test {
    use super::{Configuration, Flags};

    #[test]
    fn roundtrip_stream_vec() {
        let bits_per_sample = 8;
        let block_size = 16;
        let rsi = 32;
        let flags = Flags::DATA_MSB | Flags::DATA_PREPROCESS;
        let data = b" This is a fun message for you. ";
        assert_eq!((data.len() * 8) % (block_size * bits_per_sample), 0);

        let conf = Configuration::new(bits_per_sample, block_size, rsi, flags);
        let mut enc = conf.encoder().unwrap();
        let mut dec = conf.decoder().unwrap();

        let mut compressed = Vec::with_capacity(data.len() + 1);
        compressed.push(42);
        let mut decompressed = Vec::with_capacity(data.len() + 1);
        decompressed.push(42);

        let (unused, _) = enc.encode_vec(data, &mut compressed, true).unwrap();
        assert_eq!(unused.len(), 0);
        enc.end().unwrap();

        let (unused, _) = dec
            .decode_vec(&compressed[1..], &mut decompressed, true)
            .unwrap();
        assert_eq!(unused.len(), 0);
        dec.end().unwrap();

        assert_eq!(compressed[0], 42);
        assert_eq!(decompressed[0], 42);
        assert_eq!(&decompressed[1..], data);
    }

    #[test]
    fn roundtrip_buffer() {
        let bits_per_sample = 8;
        let block_size = 16;
        let rsi = 32;
        let flags = Flags::DATA_MSB | Flags::DATA_PREPROCESS;
        let data = b" This is a fun message for you. ";
        assert_eq!((data.len() * 8) % (block_size * bits_per_sample), 0);

        let conf = Configuration::new(bits_per_sample, block_size, rsi, flags);
        let mut encoded = vec![];
        let mut decoded = vec![];
        conf.encode_buffer(data, &mut encoded).unwrap();
        conf.decode_buffer(&encoded, &mut decoded).unwrap();
        assert_eq!(decoded, data);
    }
}
