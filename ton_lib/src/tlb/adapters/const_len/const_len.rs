
use std::ops::{Deref, DerefMut};

/// ConstLen - length is known at compile time. It's not supposed to be used directly.
///
/// use `#[tlb_derive(bits_len = {BITS_LEN})]` instead
#[derive(Debug, Clone, PartialEq)]
pub struct ConstLen<T, const BITS_LEN: u32>(pub T);

impl<T, const L: u32> Deref for ConstLen<T, L> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T, const L: u32> DerefMut for ConstLen<T, L> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tlb::tlb_type::TLBType;

    #[test]
    fn test_const_len() -> anyhow::Result<()> {
        let obj = ConstLen::<_, 24>(1);
        let cell = obj.to_cell()?;
        assert_eq!(&cell.data, &[0, 0, 1]);
        let parsed = ConstLen::<u32, 24>::from_cell(&cell)?;
        assert_eq!(obj, parsed);
        Ok(())
    }
}
