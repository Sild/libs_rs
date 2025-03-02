// use std::sync::Arc;
// use crate::cell::cell_owned::CellOwned;
// use crate::cell::ton_cell::TonCell;
// use crate::build_parse::cell_builder::TonCellBuilder;
// use crate::build_parse::cell_parser::TonCellParser;
// use crate::errors::TonLibResult;
// use crate::tlb::tlb_object::TLBObject;
//
// impl TLBObject for CellOwned {
//     fn read(parser: &mut TonCellParser) -> TonLibResult<Self> {
//
//         if parser.cell.get_data_bits_len() == parser.data_bits_left()? as usize && parser.next_ref_pos == 0 {
//             Ok(parser.cell.clone())
//         } else {
//             // TODO not clear how to handle exotics with current implementation
//             parser.load_remaining()
//         }
//     }
//
//     fn write(&self, builder: &mut TonCellBuilder) -> TonLibResult<()> {
//         builder.set_cell_is_exotic(self.is_exotic());
//         builder.store_cell(self)?;
//         Ok(())
//     }
//
//     fn from_boc(boc: &[u8]) -> Result<Self, TonCellError> {
//         let arc_cell = BagOfCells::parse(boc)?.single_root()?;
//         let cell = match Arc::try_unwrap(arc_cell) {
//             Ok(cell) => cell,
//             Err(arc_cell) => {
//                 // we just constructed the cell, so this should never happen
//                 panic!("Failed to unwrap Arc: {arc_cell:?}")
//             }
//         };
//         Ok(cell)
//     }
// }
//
// impl TLBObject for ArcCell {
//     fn read(parser: &mut CellParser) -> Result<Self, TonCellError> {
//         Cell::read(parser).map(Arc::new)
//     }
//
//     fn write(&self, builder: &mut CellBuilder) -> Result<(), TonCellError> {
//         self.as_ref().write_to(builder)?;
//         Ok(())
//     }
//
//     fn from_boc(boc: &[u8]) -> Result<Self, TonCellError> {
//         BagOfCells::parse(boc)?.single_root()
//     }
// }
//
// impl TLBObject for TonHash {
//     fn read(parser: &mut CellParser) -> Result<Self, TonCellError> {
//         let byes = parser.load_bytes(TON_HASH_LEN)?;
//         Ok(TonHash::try_from(byes)?)
//     }
//
//     fn write(&self, builder: &mut CellBuilder) -> Result<(), TonCellError> {
//         builder.store_bits(TON_HASH_LEN * 8, self.as_slice())?;
//         Ok(())
//     }
// }
