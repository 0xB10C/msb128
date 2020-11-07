![crates.io](https://img.shields.io/crates/v/msb128.svg)
[![Docs badge]][docs.rs]
[![Build Status](https://travis-ci.com/0xB10C/msb128.svg?branch=main)](https://travis-ci.com/0xB10C/msb128)

[Docs badge]: https://img.shields.io/badge/docs.rs-rustdoc-green
[docs.rs]: https://docs.rs/msb128/

# msb128

std::io::{Read, Write} **positive**, primitive Rust integers in the Most
Significant Base 128 (MSB128) variable-length encoding.

MSB128 is also known as [Variable Length Quantity] (VLQ) encoding and similar
to the [Little Endian Base 128] (LEB128) encoding (other endianness).

[Variable Length Quantity]: https://en.wikipedia.org/wiki/Variable-length_quantity
[Little Endian Base 128]: https://en.wikipedia.org/wiki/LEB128

Each byte is encoded into 7 bits, and one is subtracted (excluding the last
byte). The highest bit indicates if more bytes follow. Reading stops after a
byte with the highest bit set is read or if the underlying Rust primitive
overflows.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
