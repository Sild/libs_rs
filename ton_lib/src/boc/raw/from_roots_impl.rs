use crate::boc::raw::boc_raw::{BOCRaw, CellRaw};
use crate::cell::ton_cell::TonCellRef;
use crate::cell::ton_hash::TonHash;
use crate::errors::TonLibError;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, Clone)]
struct CellIndexed<'a> {
    cell: &'a TonCellRef,
    index: RefCell<usize>, // internal mutability required
}

impl BOCRaw {
    pub(crate) fn from_roots(roots: &[TonCellRef]) -> Result<Self, TonLibError> {
        let cell_by_hash = build_and_verify_index(roots);

        // Sort indexed cells by their index value.
        let mut cell_sorted: Vec<_> = cell_by_hash.values().collect();
        cell_sorted.sort_unstable_by(|a, b| a.index.cmp(&b.index));

        // Remove gaps in indices.
        cell_sorted
            .iter()
            .enumerate()
            .for_each(|(real_index, indexed_cell)| *indexed_cell.index.borrow_mut() = real_index);

        let raw_cells = cell_sorted
            .into_iter()
            .map(|indexed| raw_from_indexed(indexed.cell, &cell_by_hash))
            .collect::<Result<_, TonLibError>>()?;

        let root_indices = roots.iter().map(|x| get_position(x, &cell_by_hash)).collect::<Result<_, TonLibError>>()?;

        Ok(BOCRaw {
            cells: raw_cells,
            roots: root_indices,
        })
    }
}

fn build_and_verify_index(roots: &[TonCellRef]) -> HashMap<TonHash, CellIndexed> {
    let mut cur_cells = vec![];
    for cell in roots {
        cur_cells.push(cell);
    }
    let mut new_hash_index = 0;
    let mut cells_by_hash = HashMap::new();

    // Process cells to build the initial index.
    while !cur_cells.is_empty() {
        let mut next_cells = Vec::with_capacity(cur_cells.len() * 4);
        for cell in cur_cells {
            let hash = cell.hash();

            if cells_by_hash.contains_key(hash) {
                continue; // Skip if already indexed.
            }

            let indexed_cell = CellIndexed {
                cell,
                index: RefCell::new(new_hash_index),
            };
            cells_by_hash.insert(hash.clone(), indexed_cell);

            new_hash_index += 1;
            for ref_pos in 0..cell.refs_count() {
                next_cells.push(cell.get_ref(ref_pos).unwrap());
            }
        }

        cur_cells = next_cells;
    }

    // Ensure indices are in the correct order based on cell references.
    let mut verify_order = true;
    while verify_order {
        verify_order = false;

        for index_cell in cells_by_hash.values() {
            for ref_pos in 0..index_cell.cell.refs_count() {
                let ref_cell = index_cell.cell.get_ref(ref_pos).unwrap();
                let ref_hash = ref_cell.hash();
                if let Some(indexed) = cells_by_hash.get(ref_hash) {
                    if indexed.index < index_cell.index {
                        *indexed.index.borrow_mut() = new_hash_index;
                        new_hash_index += 1;
                        verify_order = true; // Verify if an index was updated.
                    }
                }
            }
        }
    }

    cells_by_hash
}

fn raw_from_indexed(cell: &TonCellRef, cells_by_hash: &HashMap<TonHash, CellIndexed>) -> Result<CellRaw, TonLibError> {
    let refs_positions = raw_cell_refs_indexes(cell, cells_by_hash)?;
    Ok(CellRaw {
        cell_type: cell.get_meta().cell_type,
        data: cell.get_data().to_vec(),
        data_bits_len: cell.get_data_bits_len(),
        refs_positions,
        level_mask: cell.get_meta().level_mask,
    })
}

fn raw_cell_refs_indexes(
    cell: &TonCellRef,
    cells_by_hash: &HashMap<TonHash, CellIndexed>,
) -> Result<Vec<usize>, TonLibError> {
    let mut vec = Vec::with_capacity(cell.refs_count());
    for ref_pos in 0..cell.refs_count() {
        let cell_ref = cell.get_ref(ref_pos).unwrap();
        vec.push(get_position(cell_ref, cells_by_hash)?);
    }
    Ok(vec)
}

fn get_position(cell: &TonCellRef, call_by_hash: &HashMap<TonHash, CellIndexed>) -> Result<usize, TonLibError> {
    let hash = cell.hash();
    call_by_hash
        .get(hash)
        .ok_or_else(|| TonLibError::BocCustom(format!("cell with hash {hash:?} not found in available hashes")))
        .map(|indexed_cell| *indexed_cell.index.borrow().deref())
}
