// use std::ops::{Deref, DerefMut};
// use crate::cell_build_parse::cell_builder::TonCellBuilder;
// use crate::cell_build_parse::cell_parser::TonCellParser;
// use crate::errors::TonLibResult;
// use crate::tlb::tlb_object::TLBObject;
//
// #[derive(Debug, PartialEq, Clone)]
// pub struct Ref<T>(pub T);
//
// impl<T> Ref<T> {
//     pub const fn new(value: T) -> Self {
//         Ref(value)
//     }
// }
//
// impl<T> Deref for Ref<T> {
//     type Target = T;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
// impl<T> DerefMut for Ref<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }
//
// impl<T: TLBObject> TLBObject for Ref<T> {
//     fn read(parser: &mut TonCellParser) -> TonLibResult<Ref<T>> {
//         Ok(Ref(T::from_cell(parser.read_next_ref()?)?))
//     }
//
//     fn write(&self, dst: &mut TonCellBuilder) -> TonLibResult<()> {
//         dst.write_ref(self.0.to_cell()?)?;
//         Ok(())
//     }
// }
//
// #[cfg(test)]
// mod test {
//     use crate::cell::ton_cell::TonCell;
//     use crate::cell_build_parse::cell_builder::TonCellBuilder;
//     use crate::tlb::primitives::reference::Ref;
//     use crate::tlb::primitives::test_types::TestType1;
//     use crate::tlb::tlb_object::TLBObject;
//
//     #[test]
//     fn test_ref() -> anyhow::Result<()> {
//         let obj = Ref::new(TestType1 { value: 1 });
//         let mut builder = TonCellBuilder::new();
//         obj.write(&mut builder)?;
//         let cell = builder.build()?;
//         assert_eq!(cell.refs_count(), 1);
//         let parsed_back = Ref::<TestType1>::from_cell(&cell)?;
//         assert_eq!(obj, parsed_back);
//         Ok(())
//     }
// }
