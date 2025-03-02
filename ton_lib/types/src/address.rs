use ton_lib_cell::cell::ton_hash::TonHash;

pub struct TonAddress {
    pub wc: i32,
    pub hash: TonHash,
}
