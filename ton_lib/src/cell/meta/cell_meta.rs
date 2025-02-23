use crate::cell::meta::cell_meta_builder::CellMetaBuilder;
use crate::cell::meta::cell_type::CellType;
use crate::cell::meta::level_mask::LevelMask;
use crate::cell::ton_cell::TonCell;
use crate::cell::ton_hash::TonHash;
use crate::errors::TonLibResult;

#[derive(Debug, Clone, PartialEq)]
pub struct CellMeta {
    pub cell_type: CellType,
    pub refs_count: usize,
    pub level_mask: LevelMask,
    pub depths: [u16; 4],
    pub hashes: [TonHash; 4],
}

impl CellMeta {
    pub const DEPTH_BYTES: usize = 2;
    pub const CELL_MAX_DATA_BITS_LEN: u32 = 1023;
    pub const CELL_MAX_REFS_COUNT: u8 = 4;

    pub const EMPTY_CELL_META: CellMeta = CellMeta {
        cell_type: CellType::Ordinary,
        refs_count: 0,
        level_mask: LevelMask::new(0),
        depths: [0; 4],
        hashes: [TonHash::EMPTY_CELL_HASH; 4],
    };

    pub fn new<T: TonCell>(
        cell_type: CellType,
        data: &[u8],
        data_bits_len: usize,
        refs: &[Option<Box<T>>; 4],
        refs_count: usize,
    ) -> TonLibResult<Self> {
        let meta_builder = CellMetaBuilder::new(cell_type, data, data_bits_len, refs, refs_count);

        // just don't look inside
        meta_builder.validate()?;
        let level_mask = meta_builder.calc_level_mask();
        let (hashes, depths) = meta_builder.calc_hashes_and_depths(level_mask)?;

        let meta = Self {
            cell_type,
            refs_count: meta_builder.refs_count,
            level_mask,
            depths,
            hashes,
        };
        Ok(meta)
    }
}
