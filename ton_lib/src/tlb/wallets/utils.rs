use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::ton_cell::TonCellRef;
use crate::errors::TonLibError;
use crate::tlb::TLBType;

pub(super) fn read_up_to_4_msgs(parser: &mut CellParser) -> Result<(Vec<u8>, Vec<TonCellRef>), TonLibError> {
    let (mut msgs_modes, mut msgs) = (vec![], vec![]);
    while let Ok(cell_ref) = parser.read_next_ref() {
        msgs.push(cell_ref.clone());
        let msg_mode = TLBType::read(parser)?;
        msgs_modes.push(msg_mode);
    }
    Ok((msgs_modes, msgs))
}

pub(super) fn write_up_to_4_msgs(
    msgs_modes: &[u8],
    msgs: &[TonCellRef],
    dst: &mut CellBuilder,
) -> Result<(), TonLibError> {
    validate_msgs_count(msgs_modes, msgs, 4)?;
    for (msg, mode) in msgs.iter().zip(msgs_modes.iter()) {
        mode.write(dst)?;
        msg.write(dst)?;
    }
    Ok(())
}

pub(super) fn validate_msgs_count(msgs_modes: &[u8], msgs: &[TonCellRef], max_cnt: usize) -> Result<(), TonLibError> {
    if msgs.len() > max_cnt || msgs_modes.len() != msgs.len() {
        let err_str = format!("wrong msgs: modes_len={}, msgs_len={}, max_len={max_cnt}", msgs_modes.len(), msgs.len());
        Err(TonLibError::TLBInvalidData(err_str))
    } else {
        Ok(())
    }
}
