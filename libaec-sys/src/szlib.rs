use libc::{c_int, c_void, size_t};

pub const SZ_ALLOW_K13_OPTION_MASK: c_int = 1;
pub const SZ_CHIP_OPTION_MASK: c_int = 2;
pub const SZ_EC_OPTION_MASK: c_int = 4;
pub const SZ_LSB_OPTION_MASK: c_int = 8;
pub const SZ_MSB_OPTION_MASK: c_int = 16;
pub const SZ_NN_OPTION_MASK: c_int = 32;
pub const SZ_RAW_OPTION_MASK: c_int = 128;

pub const SZ_OK: c_int = super::AEC_OK;
pub const SZ_OUTBUFF_FULL: c_int = 2;

pub const SZ_NO_ENCODER_ERROR: c_int = -1;
pub const SZ_PARAM_ERROR: c_int = super::AEC_CONF_ERROR;
pub const SZ_MEM_ERROR: c_int = super::AEC_MEM_ERROR;

pub const SZ_MAX_PIXELS_PER_BLOCK: c_int = 32;
pub const SZ_MAX_BLOCKS_PER_SCANLINE: c_int = 128;
pub const SZ_MAX_PIXELS_PER_SCANLINE: c_int = SZ_MAX_BLOCKS_PER_SCANLINE * SZ_MAX_PIXELS_PER_BLOCK;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct SZ_com_t {
    pub options_mask: c_int,
    pub bits_per_pixel: c_int,
    pub pixels_per_block: c_int,
    pub pixels_per_scanline: c_int,
}

extern "C" {
    pub fn SZ_BufftoBuffCompress(
        dest: *mut c_void,
        destLen: *mut size_t,
        source: *const c_void,
        sourceLen: size_t,
        param: *mut SZ_com_t,
    ) -> c_int;
    pub fn SZ_BufftoBuffDecompress(
        dest: *mut c_void,
        destLen: *mut size_t,
        source: *const c_void,
        sourceLen: size_t,
        param: *mut SZ_com_t,
    ) -> c_int;
    pub fn SZ_encoder_enabled() -> c_int;
}
