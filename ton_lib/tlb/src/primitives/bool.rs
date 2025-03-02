use crate::errors::TLBResult;
use crate::tlb_type::TLBType;
use ton_lib_cell::build_parse::builder::TonCellBuilder;
use ton_lib_cell::build_parse::parser::TonCellParser;

impl TLBType for bool {
    fn read_def(parser: &mut TonCellParser) -> TLBResult<Self> { Ok(parser.read_bit()?) }

    fn write_def(&self, builder: &mut TonCellBuilder) -> TLBResult<()> {
        builder.write_bit(*self)?;
        Ok(())
    }
}
