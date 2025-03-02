use crate::cell::meta::cell_meta::CellMeta;
use crate::cell::ton_cell::TonCell;

// Doesn't own the data - nice for reading
#[derive(Debug, Clone, Copy)]
pub struct CellSlice<'a> {
    pub meta: &'a CellMeta,
    pub data: &'a [u8],
    pub data_bits_len: usize,
    pub refs: [Option<&'a dyn TonCell>; 4],
}

impl<'a> CellSlice<'a> {
    pub fn new(
        meta: &'a CellMeta,
        data: &'a [u8],
        data_bits_len: usize,
        refs: [Option<&'a (dyn TonCell + 'a)>; 4],
    ) -> Self {
        Self {
            meta,
            data,
            data_bits_len,
            refs,
        }
    }

    pub fn from_cell(cell: &'a dyn TonCell) -> Self {
        let refs = [cell.get_ref(0), cell.get_ref(1), cell.get_ref(2), cell.get_ref(3)];
        Self::new(cell.get_meta(), cell.get_data(), cell.get_data_bits_len(), refs)
    }
}

impl TonCell for CellSlice<'_> {
    fn get_meta(&self) -> &CellMeta { self.meta }
    fn get_data(&self) -> &[u8] { self.data }
    fn get_data_bits_len(&self) -> usize { self.data_bits_len }
    fn get_ref(&self, index: usize) -> Option<&dyn TonCell> { self.refs[index] }
}
