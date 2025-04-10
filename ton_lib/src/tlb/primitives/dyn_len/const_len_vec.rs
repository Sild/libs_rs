use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::errors::TonLibError;
use crate::tlb::primitives::dyn_len::const_len::ConstLen;
use crate::tlb::TLBType;

impl<const BITS_LEN: u32> TLBType for ConstLen<Vec<u8>, BITS_LEN> {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonLibError> {
        let data: Vec<u8> = parser.read_bits(BITS_LEN)?;
        Ok(Self(data))
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> {
        builder.write_bits(&self.0, BITS_LEN)?;
        Ok(())
    }
}

// === TLBType for &Vec<u8> ===
impl<const BITS_LEN: u32> TLBType for ConstLen<&Vec<u8>, BITS_LEN> {
    fn read_definition(_parser: &mut CellParser) -> Result<Self, TonLibError> {
        unimplemented!("ConstLen::read() can't be called on ref internal type")
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> {
        builder.write_bits(self.0, BITS_LEN)?;
        Ok(())
    }
}
