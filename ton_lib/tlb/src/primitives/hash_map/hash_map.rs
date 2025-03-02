use std::collections::HashMap;
use ton_lib_cell::build_parse::builder::TonCellBuilder;
use ton_lib_cell::build_parse::parser::TonCellParser;
use crate::errors::TLBResult;
use crate::tlb_type::TLBType;

impl<K: TLBType, V: TLBType> TLBType for HashMap<K, V> {
    fn read_def(parser: &mut TonCellParser) -> TLBResult<Self> {
        if parser.read_bit()? {
            todo!()
        } else {
             Ok(HashMap::new())
        }
    }

    fn write_def(&self, dst: &mut TonCellBuilder) -> TLBResult<()> {
        if self.is_empty() {
            dst.write_bit(false)?;
            return Ok(());
        }
        dst.write_bit(true)?;
        todo!()
    }
}