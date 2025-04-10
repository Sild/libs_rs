use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::errors::TonLibError;
use crate::tlb::tlb_type::TLBType;

impl TLBType for bool {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonLibError> { parser.read_bit() }
    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> { builder.write_bit(*self) }
}
