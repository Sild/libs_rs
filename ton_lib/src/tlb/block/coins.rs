use crate::cell::ton_cell::TonCellRef;
use crate::tlb::primitives::dyn_len::var_len::VarLen;
use num_bigint::BigUint;
use std::ops::{Deref, DerefMut};
use ton_lib_proc_macro::TLBDerive;

/// https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L116
#[derive(Clone, Debug, PartialEq, TLBDerive)]
pub struct Grams(pub VarLen<BigUint, 4, true>);

/// https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L124
#[derive(Clone, Debug, PartialEq, TLBDerive)]
pub struct CurrencyCollection {
    pub grams: Grams,
    pub other: Option<TonCellRef>, // dict, but it's equal to Option<TonCellRef> in tlb format
}

impl Grams {
    pub fn new<T: Into<BigUint>>(amount: T) -> Self {
        let amount = amount.into();
        let bits_len = amount.bits() as u32;
        Self(VarLen::new(amount, bits_len))
    }
}

impl CurrencyCollection {
    pub fn new<T: Into<BigUint>>(grams: T) -> Self {
        Self {
            grams: Grams::new(grams),
            other: None,
        }
    }
}

impl Deref for Grams {
    type Target = BigUint;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for Grams {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

#[cfg(test)]
mod tests {
    use crate::tlb::block::coins::CurrencyCollection;
    use crate::tlb::TLBType;

    #[test]
    fn test_currency_collection() -> anyhow::Result<()> {
        let parsed = CurrencyCollection::from_boc_hex("b5ee9c720101010100070000094c143b1d14")?;
        assert_eq!(*parsed.grams, 3242439121u32.into());

        let cell_serial = parsed.to_cell()?;
        let parsed_back = CurrencyCollection::from_cell(&cell_serial)?;
        assert_eq!(parsed, parsed_back);
        Ok(())
    }

    #[test]
    fn test_currency_collection_zero_grams() -> anyhow::Result<()> {
        let currency = CurrencyCollection::new(0u32);
        let cell = currency.to_cell()?;
        let parsed = CurrencyCollection::from_cell(&cell)?;
        assert_eq!(*parsed.grams, 0u32.into());

        let cell_serial = parsed.to_cell()?;
        assert_eq!(cell_serial, cell);
        Ok(())
    }
}
