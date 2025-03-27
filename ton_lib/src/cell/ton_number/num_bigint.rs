use num_bigint::{BigInt, BigUint};
use num_traits::{Zero};
use crate::cell::ton_number::traits::{TonBigNumber};

impl TonBigNumber for BigInt {
    const SIGNED: bool = true;

    fn is_negative(&self) -> bool { num_traits::Signed::is_negative(self) }
    fn is_zero(&self) -> bool { Zero::is_zero(self) }

    fn min_bits_len(&self) -> u32 {
        // 1 extra bit for sign
        if Zero::is_zero(self) {
            return 2;
        }
        self.bits() as u32 + 1
    }
    
    fn to_unsigned_bytes_be(&self) -> Vec<u8> { BigInt::to_bytes_be(self).1 }

    fn from_unsigned_bytes_be(negative: bool, bytes: &[u8]) -> Self
    where
        Self: Sized
    {
        BigInt::from_signed_bytes_be(bytes)
    }

}

impl TonBigNumber for BigUint {
    const SIGNED: bool = false;

    fn is_negative(&self) -> bool { false }
    fn is_zero(&self) -> bool { Zero::is_zero(self) }

    fn min_bits_len(&self) -> u32 {
        if Zero::is_zero(self) {
            return 1;
        }
        self.bits() as u32
    }

    fn to_unsigned_bytes_be(&self) -> Vec<u8> { BigUint::to_bytes_be(self) }

    fn from_unsigned_bytes_be(negative: bool, bytes: &[u8]) -> Self
    where
        Self: Sized
    {
        BigUint::from_bytes_be(bytes)
    }
}