use crate::errors::TLBResult;
use crate::tlb_type::TLBType;
use std::ops::{Deref, DerefMut};
use ton_lib_cell::build_parse::builder::TonCellBuilder;
use ton_lib_cell::build_parse::parser::TonCellParser;
use ton_lib_cell::numbers::TonNumber;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TLBNumber<T: TonNumber, const BITS_LEN: u32>(T);

/// impl TLBNumber
impl<T: TonNumber, const BITS_LEN: u32> TLBNumber<T, BITS_LEN> {
    pub const fn new(value: T) -> Self { TLBNumber(value) }
}

/// impl TLBType
impl<T: TonNumber, const BITS_LEN: u32> TLBType for TLBNumber<T, BITS_LEN> {
    fn read_def(parser: &mut TonCellParser) -> TLBResult<Self> {
        let value = parser.read_num(BITS_LEN)?;
        Ok(TLBNumber(value))
    }

    fn write_def(&self, builder: &mut TonCellBuilder) -> TLBResult<()> {
        builder.write_num(self.0, BITS_LEN)?;
        Ok(())
    }
}

// impl From
impl<T: TonNumber, const BITS_LEN: u32> From<T> for TLBNumber<T, BITS_LEN> {
    fn from(value: T) -> Self { TLBNumber(value) }
}

/// impl Deref
impl<T: TonNumber, const BITS_LEN: u32> Deref for TLBNumber<T, BITS_LEN> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.0 }
}

/// impl DerefMut
impl<T: TonNumber, const BITS_LEN: u32> DerefMut for TLBNumber<T, BITS_LEN> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
