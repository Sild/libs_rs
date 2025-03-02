use crate::cell::meta::cell_meta::CellMeta;
use crate::cell::ton_cell::{write_cell_display, ArcTonCell, TonCell, TonCellRefsStore};
use std::fmt::Display;
use std::sync::Arc;

/// Owns the data - must be used for writing
#[derive(Debug, Clone)]
pub struct CellOwned {
    pub meta: CellMeta,
    pub data: Vec<u8>,
    pub data_bits_len: usize,
    pub refs: TonCellRefsStore,
}

impl CellOwned {
    pub const EMPTY: CellOwned = CellOwned {
        meta: CellMeta::EMPTY_CELL_META,
        data: Vec::new(),
        data_bits_len: 0,
        refs: TonCellRefsStore::new(),
    };
}

unsafe impl Sync for CellOwned {}
unsafe impl Send for CellOwned {}

impl TonCell for CellOwned {
    fn get_meta(&self) -> &CellMeta { &self.meta }
    fn get_data(&self) -> &[u8] { &self.data }
    fn get_data_bits_len(&self) -> usize { self.data_bits_len }
    fn get_refs(&self) -> &[ArcTonCell] { &self.refs }
}

impl CellOwned {
    pub fn into_ref(self) -> ArcTonCell { Arc::new(self) }
}

impl PartialEq for CellOwned {
    fn eq(&self, other: &Self) -> bool { self.hash() == other.hash() }
}

impl From<CellOwned> for ArcTonCell {
    fn from(value: CellOwned) -> Self { Arc::new(value) }
}

impl Display for CellOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write_cell_display(f, self, 0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_owned_create() {
        let child = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0x01, 0x02, 0x03],
            data_bits_len: 24,
            refs: TonCellRefsStore::new(),
        }
        .into_ref();

        let _cell = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0x04, 0x05, 0x06],
            data_bits_len: 24,
            refs: TonCellRefsStore::from([child]),
        };
    }
}
