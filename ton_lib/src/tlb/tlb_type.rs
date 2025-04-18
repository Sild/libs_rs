use crate::cell::boc::BOC;
use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::ton_cell::TonCell;
use crate::cell::ton_hash::TonHash;
use crate::errors::TonLibError;
use std::ops::Deref;

pub trait TLBType: Sized {
    const PREFIX: TLBPrefix = TLBPrefix::NULL;

    /// read-write definition
    /// https://docs.ton.org/v3/documentation/data-formats/tlb/tl-b-language#overview
    /// must be implemented by all TLB objects
    /// doesn't include prefix handling
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonLibError>;
    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonLibError>;

    /// interface - must be used by external code to read/write TLB objects
    fn read(parser: &mut CellParser) -> Result<Self, TonLibError> {
        Self::verify_prefix(parser)?;
        Self::read_definition(parser)
    }

    fn write(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> {
        Self::write_prefix(builder)?;
        self.write_definition(builder)
    }

    // Utilities
    fn cell_hash(&self) -> Result<TonHash, TonLibError> { Ok(self.to_cell()?.hash().clone()) }

    /// Reading
    fn from_cell(cell: &TonCell) -> Result<Self, TonLibError> { Self::read(&mut CellParser::new(cell)) }

    fn from_boc(boc: &[u8]) -> Result<Self, TonLibError> {
        Self::from_cell(BOC::from_bytes(boc)?.single_root()?.deref())
    }

    fn from_boc_hex<T: AsRef<[u8]>>(boc_hex: T) -> Result<Self, TonLibError> {
        Self::from_boc(&hex::decode(boc_hex.as_ref())?)
    }

    /// Writing
    fn to_cell(&self) -> Result<TonCell, TonLibError> {
        let mut builder = CellBuilder::new();
        self.write(&mut builder)?;
        builder.build()
    }

    fn to_boc(&self, add_crc32: bool) -> Result<Vec<u8>, TonLibError> {
        let mut builder = CellBuilder::new();
        self.write(&mut builder)?;
        BOC::new(builder.build()?).to_bytes(add_crc32)
    }

    fn to_boc_hex(&self, add_crc32: bool) -> Result<String, TonLibError> { Ok(hex::encode(self.to_boc(add_crc32)?)) }

    /// Helpers - mostly for internal use
    fn verify_prefix(reader: &mut CellParser) -> Result<(), TonLibError> {
        if Self::PREFIX == TLBPrefix::NULL {
            return Ok(());
        }

        let prefix_error = |given, bits_left| {
            Err(TonLibError::TLBWrongPrefix {
                exp: Self::PREFIX.value,
                given,
                bits_exp: Self::PREFIX.bits_len,
                bits_left,
            })
        };

        if reader.data_bits_left()? < Self::PREFIX.bits_len {
            return prefix_error(0, reader.data_bits_left()?);
        }

        // we handle cell_underflow above - all other errors can be rethrown
        let actual_val: u128 = reader.read_num(Self::PREFIX.bits_len)?;

        if actual_val != Self::PREFIX.value {
            reader.seek_bits(-(Self::PREFIX.bits_len as i32))?; // revert reader position
            return prefix_error(actual_val, reader.data_bits_left()?);
        }
        Ok(())
    }

    fn write_prefix(builder: &mut CellBuilder) -> Result<(), TonLibError> {
        if Self::PREFIX != TLBPrefix::NULL {
            builder.write_num(&Self::PREFIX.value, Self::PREFIX.bits_len)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TLBPrefix {
    pub value: u128,
    pub bits_len: u32,
}

impl TLBPrefix {
    pub const NULL: TLBPrefix = TLBPrefix::new(0, 0);
    pub const fn new(value: u128, bits_len: u32) -> Self { TLBPrefix { value, bits_len } }
}
