use num_bigint::BigUint;
use crate::cell::ton_cell::TonCell;
use crate::tlb::TLBType;

pub trait DictKeyAdapter<K, const KEY_BITS_LEN: usize> {
    fn make_key(key: &K) -> BigUint;
    fn extract_key(key: &BigUint) -> K;
    fn key_bits_len() -> usize {
        KEY_BITS_LEN
    }
}

pub trait DictValAdapter<V> {
    fn make_tlb<'a, TLBV: TLBType + 'a>(val: &'a V) -> TLBV;
    fn extract_tlb<TLBV: TLBType>(val: TLBV) -> V;
}

pub struct DictValAdapterNone;

impl<T: TLBType> DictValAdapter<T> for DictValAdapterNone {
    fn make_tlb<'a, TLBV: TLBType + 'a>(val: &'a T) -> TLBV {
        val
    }

    fn extract_tlb<TLBV: TLBType>(val: TLBV) -> T {
        val
    }
}

