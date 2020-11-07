//! std::io::{Read, Write} positive, primitive Rust integers in the Most
//! Significant Base 128 (MSB128) variable-length encoding.
//! MSB128 is also known as [Variable Length Quantity] (VLQ) encoding and
//! similar to the [Little Endian Base 128] (LEB128) encoding (other endianness).
//!
//! [Variable Length Quantity]: https://en.wikipedia.org/wiki/Variable-length_quantity
//! [Little Endian Base 128]: https://en.wikipedia.org/wiki/LEB128
//!
//! Each byte is encoded into 7 bits, and one is subtracted (excluding the last
//! byte). The highest bit indicates if more bytes follow. Reading stops after
//! a byte with the highest bit set is read or if the underlying Rust primitive
//! overflows.
//!

extern crate num_traits;

use std::fmt;
use std::io;

/// An error type for reading MSB128 encoded integers.
#[derive(Debug)]
pub enum ReadError {
    /// IO Error while reading.
    IoError(io::Error),
    /// Encoded integer overflowed the expected integer.
    Overflow,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            ReadError::IoError(ref e) => e.fmt(f),
            ReadError::Overflow => write!(f, "encoded integer overflows the type"),
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ReadError::IoError(ref e) => Some(e),
            ReadError::Overflow => None,
        }
    }
}

impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> Self {
        ReadError::IoError(e)
    }
}

/// Read a variable length and MSB128-encoded integer from `r`. The returned
/// integer is positive. Reading negative integers is not supported.
///
/// After a successful read, the read integer is returned.
///
/// # Errors
///
/// The interger primitive used in the function and returned by the function is
/// defined by the caller. If the integer primitive overflows while reading the
/// variable length integer, a [`ReadError::Overflow`][1] is returned.
///
/// [1]: enum.ReadError.html#variant.Overflow
///
/// # Examples
///
/// ```
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// use msb128::read_positive;
///
/// // 10, 20, 30
/// let data = [0x0A, 0x14, 0x1E];
/// let mut readable = &data[..];
///
/// assert_eq!(10i16, read_positive(&mut readable)?);
/// assert_eq!(20i8, read_positive(&mut readable)?);
/// assert_eq!(30i32, read_positive(&mut readable)?);
/// # Ok(())
/// # }
/// ```
///
/// The reader can either be passed (1) as value or (2) as mutable reference.
/// See [C-RW-VALUE](https://rust-lang.github.io/api-guidelines/interoperability.html#c-rw-value).
/// With case (1), the function returns the first variable length integer from
/// the data on each call. With the mutable reader reference from case (2), 
/// successive calls return the next value each time. Case (2) is the standard
/// reader use-case.
///
/// ```rust
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// use msb128::read_positive;
///
/// let data = [
///     0x0D,       // 13
///     0x7F,       // 127
///     0x81, 0x00, // 256
///     0xFE, 0x7F  // 16383
/// ];
/// let mut readable = &data[..];
///
/// // case (1): pass by value
/// assert_eq!(0x0Du8, read_positive(readable)?);
/// assert_eq!(0x0Du8, read_positive(readable)?);
///
/// // case (2): pass by mutable reference
/// assert_eq!(0x0Du64, read_positive(&mut readable)?);
/// assert_eq!(127u8, read_positive(&mut readable)?);
/// assert_eq!(256i32, read_positive(&mut readable)?);
/// assert_eq!(16383u16, read_positive(&mut readable)?);
/// # Ok(())
/// # }
///
/// ```
pub fn read_positive<R, I>(mut reader: R) -> Result<I, ReadError>
where
    R: io::Read,
    I: num_traits::PrimInt,
{
    let mut number: I = I::zero();
    let mut buf = [0];
    loop {
        // read the next byte from r into the buffer
        reader.read_exact(&mut buf)?;
        let buffer_value: u8 = buf[0];
        // append the last 127 bits of the buffer to the number
        // (if it wouldn't overflow while doing so)
        if number > I::max_value() >> 7 {
            return Err(ReadError::Overflow);
        }
        number = (number << 7) | I::from(buffer_value & 0x7F).unwrap();
        // If the most signigicant bit is set, then another byte follows
        if buffer_value & 0x80 > 0 {
            // add 1, if anoter byte follows
            // (if it wouldn't overflow while doing so)
            if number == I::max_value() {
                return Err(ReadError::Overflow);
            }
            number = number + I::one();
        } else {
            return Ok(number);
        }
    }
}

#[test]
fn test_reading() {
    assert_eq!(0, read_positive(&mut &[0x00][..]).unwrap());
    assert_eq!(1, read_positive(&mut &[0x01][..]).unwrap());
    assert_eq!(127, read_positive(&mut &[0x7F][..]).unwrap());
    assert_eq!(128, read_positive(&mut &[0x80, 0x00][..]).unwrap());
    assert_eq!(255, read_positive(&mut &[0x80, 0x7F][..]).unwrap());
    assert_eq!(256, read_positive(&mut &[0x81, 0x00][..]).unwrap());
    assert_eq!(16383, read_positive(&mut &[0xFE, 0x7F][..]).unwrap());
    assert_eq!(16384, read_positive(&mut &[0xFF, 0x00][..]).unwrap());
    assert_eq!(16511, read_positive(&mut &[0xFF, 0x7F][..]).unwrap());
    assert_eq!(65535, read_positive(&mut &[0x82, 0xFE, 0x7F][..]).unwrap());
    assert_eq!(
        1u64 << 32,
        read_positive(&mut &[0x8E, 0xFE, 0xFE, 0xFF, 0x00][..]).unwrap()
    );
}

/// An error type for writing MSB128 encoded integers.
#[derive(Debug)]
pub enum WriteError {
    /// IO Error while writing.
    IoError(io::Error),
    /// Passed integer is negative. Only positive (but both signed or unsigned)
    /// are allowed.
    Negative,
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            WriteError::IoError(ref e) => e.fmt(f),
            WriteError::Negative => write!(f, "writing a negative integer is unsupported"),
        }
    }
}

impl std::error::Error for WriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            WriteError::IoError(ref e) => Some(e),
            WriteError::Negative => None,
        }
    }
}

impl From<io::Error> for WriteError {
    fn from(e: io::Error) -> Self {
        WriteError::IoError(e)
    }
}

/// Write `val` to the `std::io::Write` stream `w` as an MSB128-encoded
/// integer.
///
/// # Errors
/// Only positive integers are supported. A negative input causes the
/// function to return with a [`WriteError::Negative`][1].
///
/// [1]: enum.WriteError.html#variant.Negative
///
/// # Returns
/// After a successful write, the number of bytes written to `w` is returned.
///
/// # Examples
///
/// Writing a u8 and an i128 into three bytes.
///
/// ```
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// use msb128::{write_positive, read_positive};
///
/// let mut buffer = [0u8; 3];
/// let mut writeable = &mut buffer[..];
///
/// let bytes_written = write_positive(&mut writeable, 127u8)?;
/// assert_eq!(bytes_written, 1);
///
/// let bytes_written = write_positive(&mut writeable, 256i128)?;
/// assert_eq!(bytes_written, 2);
///
/// let mut readable = &buffer[..];
/// assert_eq!(127u8, read_positive(&mut readable)?);
/// assert_eq!(256u16, read_positive(&mut readable)?);
/// # Ok(())
/// }
/// ```
pub fn write_positive<W, I>(mut writer: W, input: I) -> Result<usize, WriteError>
where
    W: io::Write,
    I: num_traits::PrimInt,
{
    // dont allow writing of negative values
    if input < I::zero() {
        return Err(WriteError::Negative);
    }
    let mut val = input.clone();
    let mut tmp = std::vec::Vec::new();
    let mut index = 0;
    loop {
        let b = (val & I::from(0x7Fu8).unwrap())
            | (if index > 0 {
                I::from(0x80).unwrap()
            } else {
                I::zero()
            });
        tmp.push(b.to_u8().unwrap());
        if val <= I::from(0x7Fu8).unwrap() {
            break;
        }
        val = (val >> 7) - I::one();
        index += 1;
    }
    tmp.reverse();
    writer.write_all(tmp.as_slice())?;
    Ok(tmp.len())
}

#[test]
fn test_writing() {
    let testcases = vec![
        (0, 1, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (1, 1, [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (127, 1, [0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (128, 2, [0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (255, 2, [0x80, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (256, 2, [0x81, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (16383, 2, [0xFE, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (16384, 2, [0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (16511, 2, [0xFF, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (65535, 3, [0x82, 0xFE, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x00]),
        (
            1i64 << 32,
            5,
            [0x8E, 0xFE, 0xFE, 0xFF, 0x00, 0x00, 0x00, 0x00],
        ),
    ];

    for tc in testcases {
        let mut buf = [0u8; 8];
        // check the length of the written data
        assert_eq!(tc.1, write_positive(&mut buf[..], tc.0).unwrap());
        // check the contents of the written data
        assert_eq!(tc.2, buf);
    }
}

#[test]
fn test_write_and_then_read() {
    let mut buf = [0u8; 4096];

    let mut testcases = vec![];
    for i in 2..128 {
        testcases.push((1u128 << i) - 1);
        testcases.push(1u128 << i);
        testcases.push((1u128 << i) + 1);
    }

    // write testcases into buf
    let mut writable = &mut buf[..];
    for tc in testcases.clone() {
        write_positive(&mut writable, tc).unwrap();
    }

    // read testcases from buf and check
    let mut readable = &buf[..];
    for tc in testcases {
        let val: u128 = read_positive(&mut readable).unwrap();
        assert_eq!(tc, val);
    }
}

#[test]
fn test_is_err_on_negative_write() {
    let mut buf = [0u8; 8];
    let mut writable = &mut buf[..];
    assert!(write_positive(&mut writable, -2).is_err());
}
