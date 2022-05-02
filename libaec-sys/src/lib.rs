//! This crate provides low-level bindings to [libaec][], the Adaptive
//! Entropy Coding library.
//!
//! [libaec]: https://gitlab.dkrz.de/k202009/libaec
//!
//! Libaec implements extended Golomb-Rice coding as defined in the
//! CCSDS recommended standard [121.0-B-3][]. The library covers the
//! adaptive entropy coder and the preprocessor discussed in sections
//! 1 to 5.2.6 of the standard.
//!
//! [121.0-B-3]: https://public.ccsds.org/Pubs/121x0b3.pdf

use libc::{c_int, c_uchar, c_uint, size_t};

pub mod szlib;

#[repr(C)]
#[derive(Debug)]
#[doc(hidden)]
pub struct internal_state {
    private: [u8; 0],
}

/// Stream configuration and state.
///
/// Set these fields appropriately, and then call [`aec_encode_init`]
/// or [`aec_decode_init`] to initialize the coder.
#[repr(C)]
#[derive(Debug)]
pub struct aec_stream {
    /// a pointer to the next input
    pub next_in: *const c_uchar,
    /// number of bytes available at [`Self::next_in`]
    pub avail_in: size_t,
    /// total number of input bytes read so far
    pub total_in: size_t,

    /// a pointer where to store the next output
    pub next_out: *mut c_uchar,
    /// remaining free space at [`Self::next_out`]
    pub avail_out: size_t,
    /// total number of bytes output so far
    pub total_out: size_t,

    /// resolution in bits per sample (n = 1, ..., 32)
    pub bits_per_sample: c_uint,
    /// block size in samples
    pub block_size: c_uint,
    /// Reference interval sample, the number of blocks
    /// between consecutive reference samples (up to 4096)
    pub rsi: c_uint,
    /// flags to control the algorithm
    pub flags: c_uint,
    /// private internal state, set to [`std::ptr::null_mut`] to initialize
    pub state: *mut internal_state,
}

// Sample data description flags

/// (flags) Samples are signed. Telling libaec this results in a
/// slightly better compression ratio. Default is unsigned.
pub const AEC_DATA_SIGNED: c_uint = 1;
/// (flags) 24 bit samples are coded in 3 bytes
pub const AEC_DATA_3BYTE: c_uint = 2;
/// (flags) Samples are stored with their most significant bit
/// first. This has nothing to do with the endianness of the
/// host. Default is LSB.
pub const AEC_DATA_MSB: c_uint = 4;
/// (flags) Set if the preprocessor should be used
pub const AEC_DATA_PREPROCESS: c_uint = 8;
/// (flags) Use restricted set of code options
pub const AEC_RESTRICTED: c_uint = 16;
/// (flags) Pad RSI to byte boundary. Only used for decoding some
/// CCSDS sample data. Do not use this to produce new data as it
/// violates the standard.
pub const AEC_PAD_RSI: c_uint = 32;
/// (flags) Do not enforce standard regarding legal block sizes.
pub const AEC_NOT_ENFORCE: c_uint = 64;

// Return codes of library functions.

/// (error) no error occurred
pub const AEC_OK: c_int = 0;
/// (error) bad configuration
pub const AEC_CONF_ERROR: c_int = -1;
/// (error) stream flushed more than once
pub const AEC_STREAM_ERROR: c_int = -2;
/// (error) bad input data
pub const AEC_DATA_ERROR: c_int = -3;
/// (error) out of memory, or [`aec_stream::next_out`] is not a
/// multiple of the storage size
pub const AEC_MEM_ERROR: c_int = -4;

// Options for flushing.

/// (flush) Do not enforce output flushing. More input may be provided
/// with later calls. So far only relevant for encoding.
pub const AEC_NO_FLUSH: c_int = 0;
/// (flush) Flush output and end encoding. The last call to
/// `aec_encode()` must set AEC_FLUSH to drain all output.
///
/// It is not possible to continue encoding of the same stream after it
/// has been flushed. For one, the last block may be padded zeros after
/// preprocessing. Secondly, the last encoded byte may be padded with
/// fill bits.
pub const AEC_FLUSH: c_int = 1;

extern "C" {
    // Streaming encoding and decoding functions

    /// Initialize a configured [`aec_stream`] for encoding. Returns
    /// an error code.
    pub fn aec_encode_init(strm: *mut aec_stream) -> c_int;
    /// Run the encoder, optionally flushing, and return an error code.
    ///
    /// This updates the input and output fields of the stream.
    ///
    /// `flush` should be set at the end of the input stream. Use
    /// [`AEC_FLUSH`] and [`AEC_NO_FLUSH`].
    pub fn aec_encode(strm: *mut aec_stream, flush: c_int) -> c_int;
    /// Free any memory used by the encoder. Returns an error code.
    pub fn aec_encode_end(strm: *mut aec_stream) -> c_int;

    /// Initialize a configured [`aec_stream`] for decoding. Returns
    /// an error code.
    pub fn aec_decode_init(strm: *mut aec_stream) -> c_int;
    /// Run the decoder, optionally flushing, and return an error code.
    ///
    /// This updates the input and output fields of the stream.
    ///
    /// `flush` should be set at the end of the input stream. Use
    /// [`AEC_FLUSH`] and [`AEC_NO_FLUSH`].
    pub fn aec_decode(strm: *mut aec_stream, flush: c_int) -> c_int;
    /// Free any memory used by the decoder. Returns an error code.
    pub fn aec_decode_end(strm: *mut aec_stream) -> c_int;

    // Utility functions for encoding or decoding a memory buffer.

    /// Utility to encode a buffer in one call. Returns an error code.
    ///
    /// This internally calls [`aec_encode_init`], [`aec_encode`], and
    /// [`aec_encode_end`] in sequence to encode a buffer stored
    /// entirely in memory.
    pub fn aec_buffer_encode(strm: *mut aec_stream) -> c_int;

    /// Utility to decode a buffer in one call. Returns an error code.
    ///
    /// This internally calls [`aec_decode_init`], [`aec_decode`], and
    /// [`aec_decode_end`] in sequence to encode a buffer stored
    /// entirely in memory.
    pub fn aec_buffer_decode(strm: *mut aec_stream) -> c_int;
}
