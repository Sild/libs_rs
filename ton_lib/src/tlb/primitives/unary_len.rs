use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::errors::TonLibError;
use crate::tlb::TLBType;
use std::ops::{Deref, DerefMut};

/// UnaryLen: format to store value as true-bits sequence
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryLen(pub u32);

impl UnaryLen {
    pub fn new<D: Into<u32>>(data: D) -> Self { Self(data.into()) }
}

impl From<u32> for UnaryLen {
    fn from(value: u32) -> Self { Self(value) }
}

impl Deref for UnaryLen {
    type Target = u32;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for UnaryLen {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl TLBType for UnaryLen {
    fn read_definition(parser: &mut CellParser) -> Result<UnaryLen, TonLibError> {
        let mut bits_len = 0;
        while parser.read_bit()? {
            bits_len += 1;
        }
        Ok(UnaryLen(bits_len))
    }

    fn write_definition(&self, dst: &mut CellBuilder) -> Result<(), TonLibError> {
        for _ in 0..self.0 {
            dst.write_bit(true)?;
        }
        dst.write_bit(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tlb::tlb_type::TLBType;

    #[test]
    fn test_unary_len() -> anyhow::Result<()> {
        let len = UnaryLen(6);
        let cell = len.to_cell()?;
        assert_eq!(cell.data, [0b11111100]);

        let parsed = UnaryLen::from_cell(&cell)?;
        assert_eq!(len, parsed);
        Ok(())
    }
}
