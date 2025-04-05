use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::ton_cell::TonCellRef;
use crate::cell::ton_hash::TonHash;
use crate::errors::TonLibError;
use crate::tlb::wallets::utils::{read_up_to_4_msgs, write_up_to_4_msgs};
use crate::tlb::TLBPrefix;
use crate::tlb::TLBType;
use ton_lib_proc_macro::TLBDerive;

/// Is not covered by tests and it generally means unsupported
/// WalletVersion::V1R1 | WalletVersion::V1R2 | WalletVersion::V1R3 | WalletVersion::V2R1 | WalletVersion::V2R2
#[derive(Debug, PartialEq, Clone, TLBDerive)]
pub struct WalletDataV1V2 {
    pub seqno: u32,
    pub public_key: TonHash,
}

/// https://docs.ton.org/participate/wallets/contracts#wallet-v2
#[derive(Debug, PartialEq, Clone)]
pub struct WalletExtMsgBodyV2 {
    pub msg_seqno: u32,
    pub valid_until: u32,
    pub msgs_modes: Vec<u8>,
    pub msgs: Vec<TonCellRef>,
}

impl WalletDataV1V2 {
    pub fn new(public_key: TonHash) -> Self { Self { seqno: 0, public_key } }
}

impl TLBType for WalletExtMsgBodyV2 {
    fn read_def(parser: &mut CellParser) -> Result<Self, TonLibError> {
        let _signature = parser.read_bits(64 * 8)?;
        let msg_seqno = TLBType::read(parser)?;
        let valid_until = TLBType::read(parser)?;
        let (msgs_modes, msgs) = read_up_to_4_msgs(parser)?;
        Ok(Self {
            msg_seqno,
            valid_until,
            msgs_modes,
            msgs,
        })
    }

    fn write_def(&self, dst: &mut CellBuilder) -> Result<(), TonLibError> {
        self.msg_seqno.write(dst)?;
        self.valid_until.write(dst)?;
        write_up_to_4_msgs(&self.msgs_modes, &self.msgs, dst)?;
        Ok(())
    }
}
