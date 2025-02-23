use crate::cell_build_parse::cell_builder::TonCellBuilder;
use crate::cell_build_parse::cell_parser::TonCellParser;
use crate::errors::{TonLibError, TonLibResult};

pub trait TLBObject: Sized {
    fn read(reader: &mut TonCellParser) -> TonLibResult<()>;

    fn write_to(&self, writer: &mut TonCellBuilder) -> TonLibResult<()>;

    fn prefix() -> &'static TLBPrefix { &TLBPrefix::NULL }

    // Utilities
    // fn cell_hash(&self) -> Result<TonHash, TonCellError> {
    //     Ok(self.to_cell()?.cell_hash())
    // }
    //
    // /// Parsing
    // ///
    // fn from_cell(cell: &dyn TonCell) -> TonCellResult<Self> {
    //     Self::read(&mut cell.parser())
    // }
    //
    // fn from_boc(boc: &[u8]) -> Ton<Self> {
    //     unimplemented!()
    // }
    //
    // fn from_boc_hex(boc_hex: &str) -> Result<Self, TonCellError> {
    //     unimplemented!()
    // }
    //
    // fn from_boc_b64(boc_b64: &str) -> Result<Self, TonCellError> {
    //     unimplemented!()
    // }
    //
    // /// Serialization
    // ///
    // fn to_cell(&self) -> Result<Cell, TonCellError> {
    //     unimplemented!()
    // }
    //
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
    fn verify_prefix(reader: &mut TonCellParser) -> TonLibResult<()> {
        let expected_prefix = Self::prefix();
        if expected_prefix == &TLBPrefix::NULL {
            return Ok(());
        }
        let actual_value = reader.lookup_bits(expected_prefix.bit_len)?;
        if actual_value != expected_prefix.value {
            return Err(TonLibError::TLBWrongPrefix {
                expected: expected_prefix.value,
                given: actual_value,
            });
        }
        Ok(())
    }

    // fn write_prefix(builder: &mut TonCellWriter) -> TonLibResult<()> {
    //     let prefix = Self::prefix();
    //     if prefix != &TLBPrefix::NULL {
    //         builder.store_u64(prefix.bit_len as usize, prefix.value)?;
    //     }
    //     Ok(())
    // }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TLBPrefix {
    pub bit_len: u8,
    pub value: u128,
}

impl TLBPrefix {
    pub const NULL: TLBPrefix = TLBPrefix { bit_len: 0, value: 0 };
    pub const fn new(bit_len: u8, value: u128) -> Self { Self { bit_len, value } }
}
