pub struct BoC {}
//
// impl BoC<'_> {
//     pub fn serial_bytes(cells: &[CellOwned], add_crc32: bool) -> Vec<u8> {
//         unimplemented!()
//     }
//
//     pub fn from_bytes(bytes: Vec<u8>) -> BocResult<Self> {
//         unimplemented!()
//     }
//
//     pub fn single_root(&mut self) -> BocResult<&CellSlice> {
//         if self.roots.len() != 1 {
//             return Err(BoCError::SingleRoot(self.roots.len()))
//         }
//         Ok(&self.roots[0])
//     }
// }
