use crate::cell::meta::cell_meta::CellMeta;
use crate::cell::meta::level_mask::LevelMask;
use crate::cell::ton_hash::TonHash;
use std::fmt::{Debug, Display, Formatter};

pub trait TonCell: Debug {
    // raw data access
    fn get_meta(&self) -> &CellMeta;
    fn get_data(&self) -> &[u8];
    fn get_data_bits_len(&self) -> usize;
    fn get_ref(&self, index: usize) -> Option<&dyn TonCell>;

    // wrappers over meta
    fn refs_count(&self) -> usize { self.get_meta().refs_count }
    fn hash(&self) -> &TonHash { self.hash_for_level(LevelMask::MAX_LEVEL) }
    fn hash_for_level(&self, level: LevelMask) -> &TonHash { &self.get_meta().hashes[level.mask() as usize] }
}

impl Display for dyn TonCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write_cell_display(f, self, 0) }
}

pub fn write_cell_display(f: &mut Formatter<'_>, cell: &dyn TonCell, indent_level: usize) -> std::fmt::Result {
    use std::fmt::Write;

    let indent = "    ".repeat(indent_level);
    // Generate the data display string
    let mut data_display = cell.get_data().iter().fold(String::new(), |mut res, byte| {
        let _ = write!(res, "{byte:02x}");
        res
    });
    // completion tag
    if cell.get_data_bits_len() % 8 != 0 {
        data_display.push('_');
    }

    if data_display.is_empty() {
        data_display.push_str("");
    };

    if cell.get_ref(0).is_none() {
        // Compact format for cells without references
        writeln!(
            f,
            "{}Cell {{Type: {:?}, data: [{}], bit_len: {}}}",
            indent,
            cell.get_meta().cell_type,
            data_display,
            cell.get_data_bits_len()
        )
    } else {
        // Full format for cells with references
        writeln!(
            f,
            "{}Cell x{{Type: {:?}, data: [{}], bit_len: {}, references: [",
            indent,
            cell.get_meta().cell_type,
            data_display,
            cell.get_data_bits_len()
        )?;
        let mut ref_pos = 0;
        while let Some(cell_ref) = cell.get_ref(ref_pos) {
            write_cell_display(f, cell_ref, indent_level + 1)?;
            ref_pos += 1;
        }
        writeln!(f, "{}]}}", indent)
    }
}
