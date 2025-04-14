use crate::cell::ton_hash::TonHash;
use crate::errors::TonLibError;
use num_bigint::BigUint;

pub trait DictKeyAdapter<K> {
    fn make_key(key: &K) -> BigUint;
    fn extract_key(key: &BigUint) -> Result<K, TonLibError>;
}

pub struct DictKeyAdapterTonHash;

impl DictKeyAdapter<TonHash> for DictKeyAdapterTonHash {
    fn make_key(key: &TonHash) -> BigUint { BigUint::from_bytes_le(key.as_slice()) }

    fn extract_key(key: &BigUint) -> Result<TonHash, TonLibError> {
        let mut hash_bytes = vec![0; TonHash::BYTES_LEN];
        let key_bytes = key.to_bytes_le();
        if key_bytes.len() > TonHash::BYTES_LEN {
            return Err(TonLibError::TLBDictWrongKeyLen {
                exp: TonHash::BYTES_LEN,
                got: key_bytes.len(),
                key: key.clone(),
            });
        }
        let offset = TonHash::BYTES_LEN - key_bytes.len();
        hash_bytes.as_mut_slice()[offset..].copy_from_slice(key_bytes.as_slice());
        TonHash::from_bytes(hash_bytes)
    }
}
