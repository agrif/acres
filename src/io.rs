use crate::{Decoder, Encoder};

use std::io;

#[derive(Clone, Debug)]
pub struct Reader<EncDec, T> {
    encdec: EncDec,
    inner: T,
}

impl<EncDec, T> Reader<EncDec, T> {
    pub fn new(encdec: EncDec, inner: T) -> io::BufReader<Self>
    where
        Self: io::Read,
    {
        io::BufReader::with_capacity(
            crate::DEFAULT_BUFFER_SIZE,
            Self::new_unbuffered(encdec, inner),
        )
    }

    pub fn new_unbuffered(encdec: EncDec, inner: T) -> Self {
        Self { encdec, inner }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

#[derive(Clone, Debug)]
pub struct Writer<EncDec, T> {
    encdec: EncDec,
    buffer: Vec<u8>,
    inner: T,
}

impl<EncDec, T> Writer<EncDec, T> {
    pub fn new(encdec: EncDec, inner: T) -> Self {
        Self::with_capacity(encdec, crate::DEFAULT_BUFFER_SIZE, inner)
    }

    pub fn with_capacity(encdec: EncDec, capacity: usize, inner: T) -> Self {
        Self {
            encdec,
            buffer: Vec::with_capacity(capacity),
            inner,
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> io::Read for Reader<Encoder, T>
where
    T: io::BufRead,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.encdec.is_ended() {
            return Ok(0);
        }
        let mut produced = 0;
        while produced < buf.len() {
            let input = self.inner.fill_buf()?;
            let inlen = input.len();
            if inlen == 0 && produced > 0 {
                // break here -- try to get the flush on a new buffer
                // since we cannot tell when flushing is done
                break;
            }
            let (rest, out) = self
                .encdec
                .encode(input, &mut buf[produced..], inlen == 0)?;
            let consumed = inlen - rest.len();
            self.inner.consume(consumed);
            produced += out.len();
            if inlen == 0 {
                self.encdec.end()?;
                break;
            }
        }
        Ok(produced)
    }
}

impl<T> io::Read for Reader<Decoder, T>
where
    T: io::BufRead,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.encdec.is_ended() {
            return Ok(0);
        }
        let mut produced = 0;
        while produced < buf.len() {
            let input = self.inner.fill_buf()?;
            if input.len() == 0 {
                self.encdec.end()?;
                break;
            }
            let (rest, out) = self.encdec.decode(input, &mut buf[produced..], false)?;
            let consumed = input.len() - rest.len();
            self.inner.consume(consumed);
            produced += out.len();
        }
        Ok(produced)
    }
}

impl<T> io::Write for Writer<Encoder, T>
where
    T: io::Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.encdec.is_ended() {
            return Ok(0);
        }
        let mut consumed = 0;
        while consumed < buf.len() {
            self.buffer.clear();
            let (rest, out) = self
                .encdec
                .encode_vec(&buf[consumed..], &mut self.buffer, false)?;
            if out.len() > 0 {
                self.inner.write(out)?;
            }
            consumed = buf.len() - rest.len();
        }
        Ok(consumed)
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.encdec.is_ended() {
            return Ok(());
        }
        self.buffer.clear();
        let (_, out) = self.encdec.encode_vec(&[], &mut self.buffer, true)?;
        if out.len() > 0 {
            self.inner.write(out)?;
        }
        self.encdec.end()?;
        self.inner.flush()
    }
}

impl<T> io::Write for Writer<Decoder, T>
where
    T: io::Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.encdec.is_ended() {
            return Ok(0);
        }
        let mut consumed = 0;
        while consumed < buf.len() {
            self.buffer.clear();
            let (rest, out) = self
                .encdec
                .decode_vec(&buf[consumed..], &mut self.buffer, false)?;
            if out.len() > 0 {
                self.inner.write(out)?;
            }
            consumed = buf.len() - rest.len();
        }
        Ok(consumed)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod test {
    use super::{Reader, Writer};
    use crate::{Configuration, Flags};
    use std::io;
    use std::io::{Read, Write};

    const DATA: &[u8] = b" This is a fun message for you. ";

    fn config() -> Configuration {
        let bits_per_sample = 8;
        let block_size = 16;
        let rsi = 16;
        let flags = Flags::DATA_MSB | Flags::DATA_PREPROCESS;
        assert_eq!((DATA.len() * 8) % (block_size * bits_per_sample), 0);
        Configuration::new(bits_per_sample, block_size, rsi, flags)
    }

    #[test]
    fn encode_reader() {
        let conf = config();
        let reader = io::BufReader::with_capacity(2, io::Cursor::new(DATA.to_owned()));
        let mut reader = Reader::new(conf.encoder().unwrap(), reader);
        let mut encoded = vec![];
        reader.read_to_end(&mut encoded).unwrap();
        let mut decoded = vec![];
        conf.decode_buffer(&encoded, &mut decoded).unwrap();
        assert_eq!(decoded, DATA);
    }

    #[test]
    fn decode_reader() {
        let conf = config();
        let mut encoded = vec![];
        conf.encode_buffer(DATA, &mut encoded).unwrap();
        let reader = io::BufReader::with_capacity(2, io::Cursor::new(encoded));
        let mut reader = Reader::new(conf.decoder().unwrap(), reader);
        let mut decoded = vec![];
        reader.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, DATA);
    }

    #[test]
    fn encode_writer() {
        let conf = config();
        let writer = io::Cursor::new(Vec::<u8>::new());
        let mut writer = Writer::with_capacity(conf.encoder().unwrap(), 64, writer);
        writer.write_all(DATA).unwrap();
        writer.flush().unwrap();
        let encoded = writer.into_inner().into_inner();
        let mut decoded = vec![];
        conf.decode_buffer(&encoded, &mut decoded).unwrap();
        assert_eq!(decoded, DATA);
    }

    #[test]
    fn decode_writer() {
        let conf = config();
        let mut encoded = vec![];
        conf.encode_buffer(DATA, &mut encoded).unwrap();
        let writer = io::Cursor::new(Vec::<u8>::new());
        let mut writer = Writer::with_capacity(conf.decoder().unwrap(), 2, writer);
        writer.write_all(&encoded).unwrap();
        let decoded = writer.into_inner().into_inner();
        assert_eq!(decoded, DATA);
    }
}
