use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::ton_cell_num::TonCellNum;
use crate::errors::TonLibError;
use crate::tlb::primitives::dyn_len::var_len::VarLen;
use crate::tlb::TLBType;

impl<T: TonCellNum, const L: u32, const BL: bool> TLBType for VarLen<T, L, BL> {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonLibError> {
        let len = parser.read_num(L)?;
        let bits_len = if BL { len * 8 } else { len };
        let data = parser.read_num(bits_len)?;
        Ok(Self { data, len })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> {
        builder.write_num(&self.len, L)?;
        let bits_len = if BL { self.len * 8 } else { self.len };
        builder.write_num(&self.data, bits_len)?;
        Ok(())
    }
}
