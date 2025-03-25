pub use bitstream_io::Numeric; // re-export
use std::fmt::Display;
pub use num_traits::Num;

pub trait TonNumber: Numeric + Display {
    type UnsignedType: Numeric;
    fn to_unsigned(&self) -> Self::UnsignedType;
}

pub trait TonBigNumber: Display {
    const SIGNED: bool;
    fn is_negative(&self) -> bool;
    fn min_bits_len(&self) -> u32;
    fn to_unsigned_bytes_be(&self) -> Vec<u8>;
    fn from_unsigned_bytes_be(negative: bool, bytes: &[u8]) -> Self;
}
