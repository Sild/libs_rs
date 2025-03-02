// use crate::cell_build_parse::cell_builder::TonCellBuilder;
// use crate::cell_build_parse::cell_parser::TonCellParser;
// use crate::errors::TonLibResult;
// use crate::tlb::tlb_object::TLBObject;
//
// #[derive(Debug, PartialEq)]
// pub(super) struct TestType1 {
//     pub(super) value: i32,
// }
//
// #[derive(Debug, PartialEq)]
// pub(super) struct TestType2 {
//     pub(super) value: i64,
// }
//
// impl TLBObject for TestType1 {
//     fn read(parser: &mut TonCellParser) -> TonLibResult<Self> {
//         Ok(TestType1 {
//             value: parser.read_num(32)?,
//         })
//     }
//
//     fn write(&self, dst: &mut TonCellBuilder) -> TonLibResult<()> {
//         dst.write_num(self.value, 32)?;
//         Ok(())
//     }
// }
//
// impl TLBObject for TestType2 {
//     fn read(parser: &mut TonCellParser) -> TonLibResult<Self> {
//         Ok(TestType2 {
//             value: parser.read_num(64)?,
//         })
//     }
//
//     fn write(&self, dst: &mut TonCellBuilder) -> TonLibResult<()> {
//         dst.write_num(self.value, 64)?;
//         Ok(())
//     }
// }
