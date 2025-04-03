use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::errors::TonLibError;
use crate::tlb::tlb_type::TLBType;
use std::ops::{Deref, DerefMut};

/// ConstLen - length is fixed and known at compile time
#[derive(Debug, Clone, PartialEq)]
pub struct ConstLen<T, const BITS_LEN: u32> {
    pub data: T,
}

/// VerLen: BITS_LEN_LEN specifies the number of bits used to store the length of the data
#[derive(Debug, Clone, PartialEq)]
pub struct VarLen<T, const BITS_LEN_LEN: u32> {
    pub bits_len: u32,
    pub data: T,
}

/// new
impl<T, const L: u32> ConstLen<T, L> {
    pub fn new(data: T) -> Self { Self { data } }
}

impl<T, const L: u32> VarLen<T, L> {
    pub fn new(bits_len: u32, data: T) -> Self { Self { bits_len, data } }
}

// From
impl<T, const L: u32> From<T> for ConstLen<T, L> {
    fn from(value: T) -> Self { Self { data: value } }
}

impl<T, const L: u32> From<(u32, T)> for VarLen<T, L> {
    fn from(value: (u32, T)) -> Self {
        Self {
            bits_len: value.0,
            data: value.1,
        }
    }
}

// Deref, DeferMut
impl<T, const L: u32> Deref for ConstLen<T, L> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.data }
}

impl<T, const L: u32> DerefMut for ConstLen<T, L> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.data }
}

impl<T, const L: u32> Deref for VarLen<T, L> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.data }
}

impl<T, const L: u32> DerefMut for VarLen<T, L> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.data }
}

/// Impl for Vec<u8>
impl<const BITS_LEN: u32> TLBType for ConstLen<Vec<u8>, BITS_LEN> {
    fn read_def(parser: &mut CellParser) -> Result<Self, TonLibError> {
        let data: Vec<u8> = parser.read_bits(BITS_LEN)?;
        Ok(Self { data })
    }

    fn write_def(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> {
        builder.write_bits(&self.data, BITS_LEN)?;
        Ok(())
    }
}

impl<const BITS_LEN_LEN: u32> TLBType for VarLen<Vec<u8>, BITS_LEN_LEN> {
    fn read_def(parser: &mut CellParser) -> Result<Self, TonLibError> {
        let bits_len: u32 = parser.read_num(BITS_LEN_LEN)?;
        let data: Vec<u8> = parser.read_bits(bits_len)?;
        Ok(Self { bits_len, data })
    }

    fn write_def(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> {
        builder.write_num(&self.bits_len, BITS_LEN_LEN)?;
        builder.write_bits(&self.data, self.bits_len)?;
        Ok(())
    }
}

/// Implementations for TonCellNum
macro_rules! dyn_len_num_impl {
    ($t:ty) => {
        impl<const L: u32> TLBType for ConstLen<$t, L> {
            fn read_def(parser: &mut CellParser) -> Result<Self, TonLibError> {
                let data = parser.read_num(L)?;
                Ok(Self { data })
            }

            fn write_def(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> {
                builder.write_num(&self.data, L)?;
                Ok(())
            }
        }

        impl<const L: u32> TLBType for VarLen<$t, L> {
            fn read_def(parser: &mut CellParser) -> Result<Self, TonLibError> {
                let bits_len: u32 = parser.read_num(L)?;
                let data: $t = parser.read_num(bits_len)?;
                Ok(Self { bits_len, data })
            }

            fn write_def(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> {
                builder.write_num(&self.bits_len, L)?;
                builder.write_num(&self.data, L)?;
                Ok(())
            }
        }
    };
}

dyn_len_num_impl!(i8);
dyn_len_num_impl!(i16);
dyn_len_num_impl!(i32);
dyn_len_num_impl!(i64);
dyn_len_num_impl!(i128);
dyn_len_num_impl!(u8);
dyn_len_num_impl!(u16);
dyn_len_num_impl!(u32);
dyn_len_num_impl!(u64);
dyn_len_num_impl!(u128);

#[cfg(feature = "num-bigint")]
dyn_len_num_impl!(num_bigint::BigUint);
#[cfg(feature = "num-bigint")]
dyn_len_num_impl!(num_bigint::BigInt);
