use crate::errors::{TLBError, TLBResult};
use ton_lib_cell::build_parse::builder::TonCellBuilder;
use ton_lib_cell::build_parse::parser::TonCellParser;
use ton_lib_cell::cell::cell_owned::CellOwned;
use ton_lib_cell::cell::ton_cell::TonCell;
use ton_lib_cell::cell::ton_hash::TonHash;

pub trait TLBType: Sized {
    // read-write definition
    // https://docs.ton.org/v3/documentation/data-formats/tlb/tl-b-language#overview
    // must be implemented by all TLB objects
    fn read_def(parser: &mut TonCellParser) -> TLBResult<Self>;
    fn write_def(&self, builder: &mut TonCellBuilder) -> TLBResult<()>;

    // interface
    fn read(parser: &mut TonCellParser) -> TLBResult<Self> {
        Self::verify_prefix(parser)?;
        Self::read_def(parser)
    }

    fn write(&self, builder: &mut TonCellBuilder) -> TLBResult<()> {
        Self::write_prefix(builder)?;
        self.write_def(builder)
    }

    fn prefix() -> &'static TLBPrefix { &TLBPrefix::NULL }

    // Utilities
    fn cell_hash(&self) -> TLBResult<TonHash> { Ok(self.to_cell()?.hash().clone()) }

    /// Parsing
    ///
    fn from_cell(cell: &dyn TonCell) -> TLBResult<Self> { Self::read(&mut TonCellParser::new(cell)) }

    // fn from_boc(boc: &[u8]) -> Ton<Self> {
    //     unimplemented!()
    // }
    //
    // fn from_boc_hex(boc_hex: &str) -> TonLibResult<Self> {
    //     unimplemented!()
    // }
    //
    // fn from_boc_b64(boc_b64: &str) -> TonLibResult<Self> {
    //     unimplemented!()
    // }

    /// Serialization
    ///
    fn to_cell(&self) -> TLBResult<CellOwned> {
        let mut builder = TonCellBuilder::new();
        self.write(&mut builder)?;
        Ok(builder.build()?)
    }

    // fn to_boc(&self, add_crc32: bool) -> Result<Vec<u8>, TonCellError> {
    //     unimplemented!()
    // }
    //
    // fn to_boc_hex(&self, add_crc32: bool) -> Result<String, TonCellError> {
    //     unimplemented!()
    // }
    //
    // fn to_boc_b64(&self, add_crc32: bool) -> Result<String, TonCellError> {
    //     unimplemented!()
    // }

    /// Helpers - for internal use
    ///
    fn verify_prefix(reader: &mut TonCellParser) -> TLBResult<()> {
        let expected_prefix = Self::prefix();
        if expected_prefix == &TLBPrefix::NULL {
            return Ok(());
        }
        let actual_value = reader.lookup_bits(expected_prefix.bit_len as u8)?;
        if actual_value != expected_prefix.value {
            return Err(TLBError::WrongPrefix {
                expected: expected_prefix.value,
                actual: actual_value,
            });
        }
        Ok(())
    }

    fn write_prefix(builder: &mut TonCellBuilder) -> TLBResult<()> {
        let prefix = Self::prefix();
        if prefix != &TLBPrefix::NULL {
            builder.write_num(prefix.value, prefix.bit_len)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TLBPrefix {
    pub value: u128,
    pub bit_len: u32,
}

impl TLBPrefix {
    pub const NULL: TLBPrefix = TLBPrefix { bit_len: 0, value: 0 };
    pub const fn new(value: u128, bit_len: u32) -> Self { Self { bit_len, value } }
}
