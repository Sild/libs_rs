use crate::errors::TLBResult;
use crate::tlb_type::TLBType;
use ton_lib_cell::build_parse::builder::TonCellBuilder;
use ton_lib_cell::build_parse::parser::TonCellParser;

#[derive(Debug, PartialEq)]
pub(super) struct TestType1 {
    pub(super) value: i32,
}

#[derive(Debug, PartialEq)]
pub(super) struct TestType2 {
    pub(super) value: i64,
}

impl TLBType for TestType1 {
    fn read_def(parser: &mut TonCellParser) -> TLBResult<Self> {
        Ok(TestType1 {
            value: parser.read_num(32)?,
        })
    }

    fn write_def(&self, dst: &mut TonCellBuilder) -> TLBResult<()> {
        dst.write_num(self.value, 32)?;
        Ok(())
    }
}

impl TLBType for TestType2 {
    fn read_def(parser: &mut TonCellParser) -> TLBResult<Self> {
        Ok(TestType2 {
            value: parser.read_num(64)?,
        })
    }

    fn write_def(&self, dst: &mut TonCellBuilder) -> TLBResult<()> {
        dst.write_num(self.value, 64)?;
        Ok(())
    }
}
