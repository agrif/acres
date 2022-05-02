//! A drop-in replacement for the SZIP library.
//!
//! This module provides low-level bindings to libaec's built-in
//! replacement for the [SZIP library][].
//!
//! [SZIP library]: http://www.hdfgroup.org/doc_resource/SZIP/

use libc::{c_int, c_void, size_t};

/// (option)
pub const SZ_ALLOW_K13_OPTION_MASK: c_int = 1;
/// (option)
pub const SZ_CHIP_OPTION_MASK: c_int = 2;
/// (option)
pub const SZ_EC_OPTION_MASK: c_int = 4;
/// (option)
pub const SZ_LSB_OPTION_MASK: c_int = 8;
/// (option)
pub const SZ_MSB_OPTION_MASK: c_int = 16;
/// (option)
pub const SZ_NN_OPTION_MASK: c_int = 32;
/// (option)
pub const SZ_RAW_OPTION_MASK: c_int = 128;

/// (error) no error occurred
pub const SZ_OK: c_int = super::AEC_OK;
/// (error) output buffer is full
pub const SZ_OUTBUFF_FULL: c_int = 2;

/// (error) encoder not available
pub const SZ_NO_ENCODER_ERROR: c_int = -1;
/// (error) bad parameters
pub const SZ_PARAM_ERROR: c_int = super::AEC_CONF_ERROR;
/// (error) out of memory
pub const SZ_MEM_ERROR: c_int = super::AEC_MEM_ERROR;

/// (limit) maximum number of pixels per block
pub const SZ_MAX_PIXELS_PER_BLOCK: c_int = 32;
/// (limit) maximum number of blocks per scanline
pub const SZ_MAX_BLOCKS_PER_SCANLINE: c_int = 128;
/// (limit) maximum number of pixels per scanline
pub const SZ_MAX_PIXELS_PER_SCANLINE: c_int = SZ_MAX_BLOCKS_PER_SCANLINE * SZ_MAX_PIXELS_PER_BLOCK;

/// common szlib compression and decompression parameters
#[repr(C)]
#[derive(Clone, Debug)]
pub struct SZ_com_t {
    pub options_mask: c_int,
    pub bits_per_pixel: c_int,
    pub pixels_per_block: c_int,
    pub pixels_per_scanline: c_int,
}

extern "C" {
    /// compress a whole buffer
    pub fn SZ_BufftoBuffCompress(
        dest: *mut c_void,
        destLen: *mut size_t,
        source: *const c_void,
        sourceLen: size_t,
        param: *mut SZ_com_t,
    ) -> c_int;

    /// decompress a whole buffer
    pub fn SZ_BufftoBuffDecompress(
        dest: *mut c_void,
        destLen: *mut size_t,
        source: *const c_void,
        sourceLen: size_t,
        param: *mut SZ_com_t,
    ) -> c_int;

    /// check if the encoder is enabled (when return value > 0)
    ///
    /// For libaec, the encoder is always enabled.
    pub fn SZ_encoder_enabled() -> c_int;
}
