use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::errors::TonLibError;
use crate::tlb::primitives::{ConstLen, VarLen};
use crate::tlb::tlb_type::TLBPrefix;
use crate::tlb::tlb_type::TLBType;
use ton_lib_proc_macro::TLBDerive;

// https://github.com/ton-blockchain/ton/blob/59a8cf0ae5c3062d14ec4c89a04fee80b5fd05c1/crypto/block/block.tlb#L100
#[derive(Debug, Clone, PartialEq, TLBDerive)]
pub enum TLBMsgAddress {
    Int(TLBMsgAddressInt),
    Ext(TLBMsgAddressExt),
}

// Ext
#[derive(Debug, Clone, PartialEq, TLBDerive)]
pub enum TLBMsgAddressExt {
    None(TLBMsgAddressNone),
    Extern(TLBMsgAddressExtern),
}

#[derive(Debug, Clone, Copy, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b00, bits_len = 2)]
pub struct TLBMsgAddressNone {}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b01, bits_len = 2)]
pub struct TLBMsgAddressExtern {
    pub address: VarLen<Vec<u8>, 9>,
}

// Int
#[derive(Debug, Clone, PartialEq, TLBDerive)]
pub enum TLBMsgAddressInt {
    Std(TLBMsgAddressIntStd),
    Var(TLBMsgAddressIntVar),
}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(prefix = 0b10, bits_len = 2)]
pub struct TLBMsgAddressIntStd {
    pub anycast: Option<Anycast>,
    pub workchain: i8,
    #[tlb_derive(bits_len = 256)]
    pub address: Vec<u8>,
}

// peculiar object - addr_bits_len is separated from addr value,
// so TLBType must be specified manually
#[derive(Debug, Clone, PartialEq)]
pub struct TLBMsgAddressIntVar {
    pub anycast: Option<Anycast>,
    pub addr_bits_len: ConstLen<u32, 9>,
    pub workchain: i32,
    pub address: Vec<u8>,
}

impl TLBType for TLBMsgAddressIntVar {
    #[rustfmt::skip]
    const PREFIX: TLBPrefix = TLBPrefix { value: 0b11, bits_len: 2};

    fn read_def(parser: &mut CellParser) -> Result<Self, TonLibError> {
        let anycast = TLBType::read(parser)?;
        let addr_bits_len: ConstLen<_, 9> = TLBType::read(parser)?;
        let workchain = TLBType::read(parser)?;
        let address = parser.read_bits(addr_bits_len.0)?;
        Ok(Self {
            anycast,
            addr_bits_len,
            workchain,
            address,
        })
    }

    fn write_def(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> {
        self.anycast.write(builder)?;
        self.addr_bits_len.write(builder)?;
        self.workchain.write(builder)?;
        builder.write_bits(&self.address, self.addr_bits_len.0)?;
        Ok(())
    }
}

/// Allows easily convert enum variants to parent type
#[rustfmt::skip]
mod from_impl {
    use crate::tlb::block::msg_address::*;
    impl From<TLBMsgAddressNone> for TLBMsgAddressExt { fn from(value: TLBMsgAddressNone) -> Self { Self::None(value) } }
    impl From<TLBMsgAddressExtern> for TLBMsgAddressExt { fn from(value: TLBMsgAddressExtern) -> Self { Self::Extern(value) } }
    impl From<TLBMsgAddressIntStd> for TLBMsgAddressInt { fn from(value: TLBMsgAddressIntStd) -> Self { Self::Std(value) } }
    impl From<TLBMsgAddressIntVar> for TLBMsgAddressInt { fn from(value: TLBMsgAddressIntVar) -> Self { Self::Var(value) } }
    impl From<TLBMsgAddressInt> for TLBMsgAddress { fn from(value: TLBMsgAddressInt) -> Self { Self::Int(value) } }
    impl From<TLBMsgAddressExt> for TLBMsgAddress { fn from(value: TLBMsgAddressExt) -> Self { Self::Ext(value) } }
    impl From<TLBMsgAddressNone> for TLBMsgAddress { fn from(value: TLBMsgAddressNone) -> Self { Self::Ext(value.into()) } }
    impl From<TLBMsgAddressExtern> for TLBMsgAddress { fn from(value: TLBMsgAddressExtern) -> Self { Self::Ext(value.into()) } }
    impl From<TLBMsgAddressIntStd> for TLBMsgAddress { fn from(value: TLBMsgAddressIntStd) -> Self { Self::Int(value.into()) } }
    impl From<TLBMsgAddressIntVar> for TLBMsgAddress { fn from(value: TLBMsgAddressIntVar) -> Self { Self::Int(value.into()) } }
}

#[derive(Debug, Clone, PartialEq, TLBDerive)]
pub struct Anycast {
    pub rewrite_pfx: VarLen<Vec<u8>, 5>,
}

impl Anycast {
    pub fn new(depth: u32, rewrite_pfx: Vec<u8>) -> Self {
        Self {
            rewrite_pfx: VarLen::new(rewrite_pfx, depth),
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio_test::assert_ok;

    use super::*;
    use crate::tlb::tlb_type::TLBType;

    #[test]
    fn test_read_write_msg_address() -> anyhow::Result<()> {
        // Anyhow read/write is covered under the hood
        let boc =
            "b5ee9c7201010101002800004bbe031053100134ea6c68e2f2cee9619bdd2732493f3a1361eccd7c5267a9eb3c5dcebc533bb6";
        let parsed = TLBMsgAddress::from_boc_hex(boc)?;
        let expected = TLBMsgAddressIntStd {
            anycast: Some(Anycast::new(30, vec![3, 16, 83, 16])),
            workchain: 0,
            address: vec![
                77, 58, 155, 26, 56, 188, 179, 186, 88, 102, 247, 73, 204, 146, 79, 206, 132, 216, 123, 51, 95, 20,
                153, 234, 122, 207, 23, 115, 175, 20, 206, 237,
            ],
        };
        assert_eq!(parsed, expected.into());

        let serial_cell = parsed.to_cell()?;
        let parsed_back = assert_ok!(TLBMsgAddress::from_cell(&serial_cell));
        assert_eq!(parsed_back, parsed);
        Ok(())
    }

    #[test]
    fn test_read_msg_address_int_i8_workchain() -> anyhow::Result<()> {
        let boc = "b5ee9c720101010100240000439fe00000000000000000000000000000000000000000000000000000000000000010";
        let parsed = assert_ok!(TLBMsgAddress::from_boc_hex(boc));

        let expected = TLBMsgAddressIntStd {
            anycast: None,
            workchain: -1,
            address: vec![0; 32],
        };
        assert_eq!(parsed, expected.into());

        // don't support same layout, so check deserialized data again
        let serial_cell = parsed.to_cell()?;
        let parsed_back = assert_ok!(TLBMsgAddress::from_cell(&serial_cell));
        assert_eq!(parsed, parsed_back);
        Ok(())
    }

    #[test]
    fn test_read_msg_address_int() -> anyhow::Result<()> {
        let boc = "b5ee9c720101010100240000439fe00000000000000000000000000000000000000000000000000000000000000010";
        let parsed = assert_ok!(TLBMsgAddressInt::from_boc_hex(boc));

        let expected = TLBMsgAddressIntStd {
            anycast: None,
            workchain: -1,
            address: vec![0; 32],
        };
        assert_eq!(parsed, expected.into());

        // don't support same layout, so check deserialized data again
        let serial_cell = parsed.to_cell()?;
        let parsed_back = assert_ok!(TLBMsgAddressInt::from_cell(&serial_cell));
        assert_eq!(parsed, parsed_back);
        Ok(())
    }
}
