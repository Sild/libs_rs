use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::ton_cell::TonCellRef;
use crate::cell::ton_hash::TonHash;
use crate::errors::TonLibError;
use crate::tlb::wallets::utils::{read_up_to_4_msgs, write_up_to_4_msgs};
use crate::tlb::TLBPrefix;
use crate::tlb::TLBType;
use ton_lib_proc_macro::TLBDerive;

/// WalletVersion::V3R1 | WalletVersion::V3R2
#[derive(Debug, PartialEq, Clone, TLBDerive)]
pub struct WalletDataV3 {
    pub seqno: u32,
    pub wallet_id: i32,
    pub public_key: TonHash,
}

/// https://docs.ton.org/participate/wallets/contracts#wallet-v3
/// signature is not considered as part of msg body
/// documentation is not correct about body layout
#[derive(Debug, PartialEq, Clone)]
pub struct WalletExtMsgBodyV3 {
    pub subwallet_id: i32,
    pub valid_until: u32,
    pub msg_seqno: u32,
    pub msgs_modes: Vec<u8>,
    pub msgs: Vec<TonCellRef>,
}

impl WalletDataV3 {
    pub fn new(wallet_id: i32, public_key: TonHash) -> Self {
        Self {
            seqno: 0,
            wallet_id,
            public_key,
        }
    }
}

impl TLBType for WalletExtMsgBodyV3 {
    fn read_def(parser: &mut CellParser) -> Result<Self, TonLibError> {
        let subwallet_id = TLBType::read(parser)?;
        let valid_until = TLBType::read(parser)?;
        let msg_seqno = TLBType::read(parser)?;
        let (msgs_modes, msgs) = read_up_to_4_msgs(parser)?;
        Ok(Self {
            subwallet_id,
            msg_seqno,
            valid_until,
            msgs_modes,
            msgs,
        })
    }

    fn write_def(&self, dst: &mut CellBuilder) -> Result<(), TonLibError> {
        self.subwallet_id.write(dst)?;
        self.valid_until.write(dst)?;
        self.msg_seqno.write(dst)?;
        write_up_to_4_msgs(&self.msgs_modes, &self.msgs, dst)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cell::ton_cell::TonCell;
    use crate::tlb::wallets::DEFAULT_WALLET_ID;
    use hex::FromHex;

    #[test]
    fn test_wallet_data_v3() -> anyhow::Result<()> {
        // https://tonviewer.com/UQAMY2B4xfQO6m3YpmzfX5Za-Ning4kWKFjPdubbPPV3Ffel
        let src_boc_hex = "b5ee9c7241010101002a0000500000000129a9a317cbf377c9b73604c70bf73488ddceba14f763baef2ac70f68d1d6032a120149f4b6de3f10";
        let wallet_data = WalletDataV3::from_boc_hex(src_boc_hex)?;
        assert_eq!(wallet_data.seqno, 1);
        assert_eq!(wallet_data.wallet_id, DEFAULT_WALLET_ID);
        assert_eq!(
            wallet_data.public_key,
            TonHash::from_hex("cbf377c9b73604c70bf73488ddceba14f763baef2ac70f68d1d6032a120149f4")?
        );
        let serial_boc_hex = wallet_data.to_boc_hex(false)?;
        let restored = WalletDataV3::from_boc_hex(&serial_boc_hex)?;
        assert_eq!(wallet_data, restored);
        Ok(())
    }

    #[test]
    fn test_wallet_ext_msg_body_v3() -> anyhow::Result<()> {
        // https://tonviewer.com/transaction/b4bd316c74b4c99586e07c167979ce4a6e18db31704abd7e85b1cacb065ce66c
        let body_signed_cell = TonCell::from_boc_hex("b5ee9c7201010201008500019a86be376ea96e2f1252377976716a3d252906151feabc8e4b51506405035e45a7b4ff81f783cfe3f86483c822bcbb4f9481804990868bac69caf7af56e30fe70b29a9a317ffffffff000000000301006642007847b4630eb08d9f486fe846d5496878556dfd5a084f82a9a3fb01224e67c84c187a120000000000000000000000000000")?;
        let mut parser = CellParser::new(&body_signed_cell);
        parser.read_bits(64 * 8)?; // signature

        let body_cell = TonCell::read(&mut parser)?;
        let body = WalletExtMsgBodyV3::from_cell(&body_cell)?;
        assert_eq!(body.subwallet_id, DEFAULT_WALLET_ID);
        assert_eq!(body.msg_seqno, 0);
        assert_eq!(body.valid_until, 4294967295);
        assert_eq!(body.msgs_modes, vec![3]);
        assert_eq!(body.msgs.len(), 1);

        let serial_cell = body.to_cell()?;
        assert_eq!(body_cell, serial_cell);
        Ok(())
    }
}
