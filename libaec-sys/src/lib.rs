use libc::{c_int, c_uchar, c_uint, size_t};

pub mod szlib;

#[repr(C)]
#[derive(Debug)]
pub struct internal_state {
    private: [u8; 0],
}

#[repr(C)]
#[derive(Debug)]
pub struct aec_stream {
    pub next_in: *const c_uchar,
    pub avail_in: size_t,
    pub total_in: size_t,
    pub next_out: *mut c_uchar,
    pub avail_out: size_t,
    pub total_out: size_t,
    pub bits_per_sample: c_uint,
    pub block_size: c_uint,
    pub rsi: c_uint,
    pub flags: c_uint,
    pub state: *mut internal_state,
}

pub const AEC_DATA_SIGNED: c_uint = 1;
pub const AEC_DATA_3BYTE: c_uint = 2;
pub const AEC_DATA_MSB: c_uint = 4;
pub const AEC_DATA_PREPROCESS: c_uint = 8;
pub const AEC_RESTRICTED: c_uint = 16;
pub const AEC_PAD_RSI: c_uint = 32;
pub const AEC_NOT_ENFORCE: c_uint = 64;

pub const AEC_OK: c_int = 0;
pub const AEC_CONF_ERROR: c_int = -1;
pub const AEC_STREAM_ERROR: c_int = -2;
pub const AEC_DATA_ERROR: c_int = -3;
pub const AEC_MEM_ERROR: c_int = -4;

pub const AEC_NO_FLUSH: c_int = 0;
pub const AEC_FLUSH: c_int = 1;

extern "C" {
    pub fn aec_encode_init(strm: *mut aec_stream) -> c_int;
    pub fn aec_encode(strm: *mut aec_stream, flush: c_int) -> c_int;
    pub fn aec_encode_end(strm: *mut aec_stream) -> c_int;

    pub fn aec_decode_init(strm: *mut aec_stream) -> c_int;
    pub fn aec_decode(strm: *mut aec_stream, flush: c_int) -> c_int;
    pub fn aec_decode_end(strm: *mut aec_stream) -> c_int;

    pub fn aec_buffer_encode(strm: *mut aec_stream) -> c_int;
    pub fn aec_buffer_decode(strm: *mut aec_stream) -> c_int;
}
