# acres

[![build status](https://github.com/agrif/acres/actions/workflows/build.yaml/badge.svg?branch=master)](https://github.com/agrif/acres/actions/workflows/build.yaml)
[![crates.io](https://img.shields.io/crates/v/acres.svg)](https://crates.io/crates/acres)
[![docs.rs](https://docs.rs/acres/badge.svg)](https://docs.rs/acres)

Rust bindings for [*libaec*][].

 [*libaec*]: https://gitlab.dkrz.de/k202009/libaec

*libaec* implements [Golomb-Rice][] coding as defined in the
Consultative Committee for Space Data Systems (CCSDS) standard
document [121.0-B-3][].

 [Golomb-Rice]: http://en.wikipedia.org/wiki/Golomb_coding
 [121.0-B-3]: https://public.ccsds.org/Pubs/121x0b3.pdf

*libaec* (and thus *acres*) also includes an implementation of the
[*szip*][] library.

 [*szip*]: http://www.hdfgroup.org/doc_resource/SZIP/

## License

Licensed under the [MIT license](LICENSE). Unless stated otherwise,
any contributions to this work will also be licensed this way, with no
additional terms or conditions.

*libaec* has [its own license](LICENSE.libaec).
