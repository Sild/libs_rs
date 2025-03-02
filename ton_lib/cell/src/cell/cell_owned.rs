use crate::cell::meta::cell_meta::CellMeta;
use crate::cell::ton_cell::{write_cell_display, TonCell};
use std::fmt::Display;
use std::ops::Deref;

pub type CellOwnedRefs = [Option<Box<CellOwned>>; 4];

/// Owns the data - must be used for writing
#[derive(Debug, Clone, PartialEq)]
pub struct CellOwned {
    pub meta: CellMeta,
    pub data: Vec<u8>,
    pub data_bits_len: usize,
    pub refs: CellOwnedRefs,
}

impl CellOwned {
    pub const EMPTY: CellOwned = CellOwned {
        meta: CellMeta::EMPTY_CELL_META,
        data: Vec::new(),
        data_bits_len: 0,
        refs: [None, None, None, None],
    };
}

impl TonCell for CellOwned {
    fn get_meta(&self) -> &CellMeta { &self.meta }
    fn get_data(&self) -> &[u8] { &self.data }
    fn get_data_bits_len(&self) -> usize { self.data_bits_len }
    fn get_ref(&self, index: usize) -> Option<&dyn TonCell> {
        self.refs[index].as_ref().map(|x| x.deref() as &dyn TonCell)
    }
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
            refs: [None, None, None, None],
        };

        let _cell = CellOwned {
            meta: CellMeta::EMPTY_CELL_META,
            data: vec![0x04, 0x05, 0x06],
            data_bits_len: 24,
            refs: [Some(Box::new(child)), None, None, None],
        };
    }
}
