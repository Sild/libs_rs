use crate::errors::TonTypesResult;
use ton_lib_cell::cell::ton_cell::ArcTonCell;
use ton_lib_cell::cell::ton_hash::TonHash;
use ton_lib_tlb::block::state_init::StateInit;
use ton_lib_tlb::tlb_type::TLBType;

pub struct TonAddress {
    pub wc: i32,
    pub hash: TonHash,
}

impl TonAddress {
    pub fn new(wc: i32, hash: TonHash) -> Self { Self { wc, hash } }
    pub fn derive(wc: i32, code: ArcTonCell, data: ArcTonCell) -> TonTypesResult<TonAddress> {
        let state_init = StateInit::new(code, data);
        Ok(TonAddress::new(wc, state_init.cell_hash()?))
    }
}
//
// #[cfg(test)]
// mod tests {
//     #[test]
//     fn test_ton_address_derive_stonfi_pool() -> anyhow::Result<()> {
//         let code_cell = Cell::from_boc_hex("b5ee9c7201010101002300084202a9338ecd624ca15d37e4a8d9bf677ddc9b84f0e98f05f2fb84c7afe332a281b4")?;
//         let data_cell = Cell::from_boc_hex("b5ee9c720101040100b900010d000000000000050102c9801459f7c0a12bb4ac4b78a788c425ee4d52f8b6041dda17b77b09fc5a03e894d6900287cd9fbe2ea663415da0aa6bbdf0cb136abe9c4f45214dd259354b80da8c265a006aebb27f5d0f1daf43e200f52408f3eb9ff5610f5b43284224644e7c6a590d14400203084202c00836440d084e44fb94316132ac5a21417ef4f429ee09b5560b5678b334c3e8084202c95a2ed22ab516f77f9d4898dc4578e72f18a2448e8f6832334b0b4bf501bc79")?;
//         let address = TonAddress::derive(0, code_cell.to_arc(), data_cell.to_arc())?;
//         let exp_addr = TonAddress::from_str("EQAdltEfzXG_xteLFaKFGd-HPVKrEJqv_FdC7z2roOddRNdM")?;
//         assert_eq!(address, exp_addr);
//         Ok(())
//     }
// }
