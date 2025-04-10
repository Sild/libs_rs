use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::ton_cell_num::TonCellNum;
use crate::errors::TonLibError;
use crate::tlb::primitives::dyn_len::const_len::ConstLen;
use crate::tlb::TLBType;

impl<T: TonCellNum, const L: u32> TLBType for ConstLen<T, L> {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonLibError> {
        let data = parser.read_num(L)?;
        Ok(Self(data))
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> {
        builder.write_num(&self.0, L)?;
        Ok(())
    }
}
