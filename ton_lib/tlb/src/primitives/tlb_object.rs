use ton_lib_cell::cell::ton_cell::{ArcTonCell, TonCell};
use crate::tlb_type::TLBType;

pub enum TLBObject<T: TLBType, C: TonCell> {
    Cell(C),
    CellRef(ArcTonCell),
    Plain(T),
}