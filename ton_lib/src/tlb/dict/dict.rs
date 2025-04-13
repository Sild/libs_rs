// use std::collections::HashMap;
// use num_bigint::BigUint;
// use crate::tlb::dict::adapters_key::{DictKeyAdapter, DictValAdapter};
// use crate::tlb::TLBType;
// 
// pub struct Dict<K, V, KA, VA> {
//     data: HashMap<K, V>,
//     key_bits_len: usize,
//     _phantom_ka: std::marker::PhantomData<KA>,
//     _phantom_va: std::marker::PhantomData<VA>,
// }
// 
// impl<K, V, KA, VA, const KEY_BITS_LEN: usize> Dict<K, V, KA, VA>
// where
//     KA: DictKeyAdapter<K, 8>,
//     VA: DictValAdapter<V>,
// {
//     pub fn new(data: HashMap<K, V>, key_bits_len: usize, key_adapter: KeyAdapter<K>, val_adapter: ValAdapter<V, TLBV>) -> Self {
//         Self { data, key_bits_len }
//     }
// }
// 
// 
// 
// // use crate::cell::build_parse::builder::CellBuilder;
// // use crate::cell::build_parse::parser::CellParser;
// // use crate::errors::TonLibError;
// // use crate::tlb::primitives::dict::data_builder::DictDataBuilder;
// // use crate::tlb::primitives::dict::data_parser::DictDataParser;
// // use crate::tlb::primitives::dyn_len::const_len::ConstLen;
// // use crate::tlb::tlb_type::TLBType;
// // use num_bigint::BigUint;
// // use std::collections::{BTreeMap, HashMap};
// // use std::error::Error;
// // use std::fmt::Debug;
// // use std::hash::Hash;
// // // Implementations are highly inefficient.
// //
// // // macro_rules! tlb_const_len_dict_impl {
// // //     ($t:tt) => {
// // //         impl<K, V, const L: u32> TLBType for ConstLen<$t<K, V>, L>
// // //         where
// // //             K: TryFrom<BigUint> + Into<BigUint> + Clone + Ord + Hash + Debug + Error,
// // //             V: TLBType,
// // //         {
// // //             fn read_definition(parser: &mut CellParser) -> Result<Self, TonLibError> {
// // //                 if !parser.read_bit()? {
// // //                     return Ok(ConstLen($t::new()));
// // //                 }
// // //
// // //                 let data_cell = parser.read_next_ref()?;
// // //                 let mut data_parser = DictDataParser::new(L as usize);
// // //                 let data_raw = data_parser.parse::<V>(&mut CellParser::new(data_cell))?;
// // //                 let data = data_raw.into_iter().map(|(k, v)| (K::try_from(k).ok_or(panic!("123")), v)).collect::<$t<K, V>>();
// // //                 Ok(ConstLen(data))
// // //             }
// // //
// // //             fn write_definition(&self, dst: &mut CellBuilder) -> Result<(), TonLibError> {
// // //                 if self.is_empty() {
// // //                     dst.write_bit(false)?;
// // //                     return Ok(());
// // //                 }
// // //
// // //                 let mut keys = Vec::with_capacity(self.len());
// // //                 for key in self.keys() {
// // //                     keys.push(key);
// // //                 }
// // //                 keys.sort_unstable();
// // //                 let mut values_sorted = Vec::with_capacity(self.len());
// // //                 for key in &keys {
// // //                     let value = self.get(key).unwrap();
// // //                     values_sorted.push(value.clone());
// // //                 }
// // //                 let keys_sorted: Vec<BigUint> = keys.into_iter().map(|k| k.clone().into()).collect();
// // //                 let data_builder = DictDataBuilder::new(0, keys_sorted, values_sorted)?;
// // //                 let dict_data_cell = data_builder.build()?.into_ref();
// // //                 dst.write_bit(true)?;
// // //                 dst.write_ref(dict_data_cell)
// // //             }
// // //         }
// // //     };
// // // }
// // //
// // // tlb_const_len_dict_impl!(HashMap);
// // // tlb_const_len_dict_impl!(BTreeMap);
// // //
// //
// // impl<K, V, const L: u32> TLBType for ConstLen<HashMap<K, V>, L>
// // where
// //     K: TryFrom<BigUint> + Into<BigUint> + Clone + Ord + Hash,
// //     V: TLBType,
// //     <K as TryFrom<BigUint>>::Error: Debug,
// // {
// //     fn read_definition(parser: &mut CellParser) -> Result<Self, TonLibError> {
// //         if !parser.read_bit()? {
// //             return Ok(ConstLen(HashMap::new()));
// //         }
// //
// //         let data_cell = parser.read_next_ref()?;
// //         let mut data_parser = DictDataParser::new(L as usize);
// //         let data_raw = data_parser.parse::<V>(&mut CellParser::new(data_cell))?;
// //
// //         let data = data_raw.into_iter().map(|(k, v)| (K::try_from(k).unwrap(), v)).collect();
// //         Ok(ConstLen(data))
// //     }
// //
// //     fn write_definition(&self, dst: &mut CellBuilder) -> Result<(), TonLibError> {
// //         if self.is_empty() {
// //             dst.write_bit(false)?;
// //             return Ok(());
// //         }
// //
// //         let mut keys = Vec::with_capacity(self.len());
// //         for key in self.keys() {
// //             keys.push(key);
// //         }
// //         keys.sort_unstable();
// //         let mut values_sorted = Vec::with_capacity(self.len());
// //         for key in &keys {
// //             let value = self.get(key).unwrap();
// //             values_sorted.push(value.clone());
// //         }
// //         let keys_sorted: Vec<BigUint> = keys.into_iter().map(|k| k.clone().into()).collect();
// //         let data_builder = DictDataBuilder::new(0, keys_sorted, values_sorted)?;
// //         let dict_data_cell = data_builder.build()?.into_ref();
// //         dst.write_bit(true)?;
// //         dst.write_ref(dict_data_cell)
// //     }
// // }
// //
// // #[cfg(test)]
// // mod tests {
// //     use super::*;
// //
// //     #[test]
// //     fn test_blockchain_data() -> anyhow::Result<()> {
// //         let expected_data = HashMap::from([
// //             (0u8, ConstLen::<_, 150>::new(BigUint::from(25965603044000000000u128))),
// //             (1, ConstLen::<_, 150>::new(BigUint::from(5173255344000000000u64))),
// //             (2, ConstLen::<_, 150>::new(BigUint::from(344883687000000000u64))),
// //         ]);
// //         let boc = "b5ee9c7241010601005a000119c70d3ca5000d99b931ea4e8cc0010201cd020302012004050027400000000000000000000001325178b51d9180200026000000000000000000000168585a65986be8000026000000000000000000000047cb18538782e000353c80b9";
// //         let parsed: ConstLen<HashMap<u8, ConstLen<BigUint, 150>>, 8> = TLBType::from_boc_hex(boc)?;
// //         assert_eq!(expected_data, *parsed);
// //         Ok(())
// //     }
// //     //
// //     //     #[test]
// //     //     fn test_key_len_bigger_than_reader() -> anyhow::Result<()> {
// //     //         let data = HashMap::from([
// //     //             (0u16, BigUint::from(4u32)),
// //     //             (1, BigUint::from(5u32)),
// //     //             (2, BigUint::from(6u32)),
// //     //             (10u16, BigUint::from(7u32)),
// //     //             (127, BigUint::from(8u32)),
// //     //         ]);
// //     //
// //     //         for key_len_bits in [8, 16, 32, 64, 111] {
// //     //             let mut builder = CellBuilder::new();
// //     //             builder.store_dict(key_len_bits, val_writer_unsigned_min_size, data.clone())?;
// //     //             let dict_cell = builder.build()?;
// //     //             let parsed = dict_cell
// //     //                 .parser()
// //     //                 .load_dict(key_len_bits, key_reader_u16, val_reader_uint)?;
// //     //             assert_eq!(data, parsed, "key_len_bits: {}", key_len_bits);
// //     //         }
// //     //         Ok(())
// //     //     }
// //     //
// //     //     #[test]
// //     //     fn test_reader_u8() -> anyhow::Result<()> {
// //     //         let data = HashMap::from([
// //     //             (0u8, BigUint::from(4u32)),
// //     //             (1, BigUint::from(5u32)),
// //     //             (2, BigUint::from(6u32)),
// //     //             (64, BigUint::from(7u32)),
// //     //         ]);
// //     //         let key_len_bits = 8;
// //     //         let mut builder = CellBuilder::new();
// //     //         builder.store_dict(key_len_bits, val_writer_unsigned_min_size, data.clone())?;
// //     //         let dict_cell = builder.build()?;
// //     //         let parsed = dict_cell
// //     //             .parser()
// //     //             .load_dict(key_len_bits, key_reader_u8, val_reader_uint)?;
// //     //         assert_eq!(data, parsed);
// //     //         Ok(())
// //     //     }
// //     //
// //     //     #[test]
// //     //     fn test_reader_u16() -> anyhow::Result<()> {
// //     //         let data = HashMap::from([
// //     //             (0u16, BigUint::from(4u32)),
// //     //             (1, BigUint::from(5u32)),
// //     //             (2, BigUint::from(6u32)),
// //     //             (64, BigUint::from(7u32)),
// //     //         ]);
// //     //         let key_len_bits = 8;
// //     //         let mut builder = CellBuilder::new();
// //     //         builder.store_dict(key_len_bits, val_writer_unsigned_min_size, data.clone())?;
// //     //         let dict_cell = builder.build()?;
// //     //         let parsed = dict_cell
// //     //             .parser()
// //     //             .load_dict(key_len_bits, key_reader_u16, val_reader_uint)?;
// //     //         assert_eq!(data, parsed);
// //     //         Ok(())
// //     //     }
// //     //
// //     //     #[test]
// //     //     fn test_reader_u32() -> anyhow::Result<()> {
// //     //         let data = HashMap::from([
// //     //             (0u32, BigUint::from(4u32)),
// //     //             (1, BigUint::from(5u32)),
// //     //             (2, BigUint::from(6u32)),
// //     //             (64, BigUint::from(7u32)),
// //     //         ]);
// //     //         let key_len_bits = 8;
// //     //         let mut builder = CellBuilder::new();
// //     //         builder.store_dict(key_len_bits, val_writer_unsigned_min_size, data.clone())?;
// //     //         let dict_cell = builder.build()?;
// //     //         let parsed = dict_cell
// //     //             .parser()
// //     //             .load_dict(key_len_bits, key_reader_u32, val_reader_uint)?;
// //     //         assert_eq!(data, parsed);
// //     //         Ok(())
// //     //     }
// //     //
// //     //     #[test]
// //     //     fn test_reader_u64() -> anyhow::Result<()> {
// //     //         let data = HashMap::from([
// //     //             (0u64, BigUint::from(4u32)),
// //     //             (1, BigUint::from(5u32)),
// //     //             (2, BigUint::from(6u32)),
// //     //             (64, BigUint::from(7u32)),
// //     //         ]);
// //     //         let key_len_bits = 8;
// //     //         let mut builder = CellBuilder::new();
// //     //         builder.store_dict(key_len_bits, val_writer_unsigned_min_size, data.clone())?;
// //     //         let dict_cell = builder.build()?;
// //     //         let parsed = dict_cell
// //     //             .parser()
// //     //             .load_dict(key_len_bits, key_reader_u64, val_reader_uint)?;
// //     //         assert_eq!(data, parsed);
// //     //         Ok(())
// //     //     }
// //     //
// //     //     #[test]
// //     //     fn test_reader_256bit() -> anyhow::Result<()> {
// //     //         let bytes1 = TonHash::from([
// //     //             1u8, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4,
// //     //             4, 4,
// //     //         ]);
// //     //         let bytes2 = TonHash::from([
// //     //             2u8, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5,
// //     //             5, 5,
// //     //         ]);
// //     //         let bytes3 = TonHash::from([
// //     //             3u8, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6,
// //     //             6, 6,
// //     //         ]);
// //     //         let bytes4 = TonHash::from([
// //     //             4u8, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7,
// //     //             7, 7,
// //     //         ]);
// //     //
// //     //         let data_src = HashMap::from([
// //     //             (bytes1, BigUint::from(1u32)),
// //     //             (bytes2, BigUint::from(2u32)),
// //     //             (bytes3, BigUint::from(3u32)),
// //     //             (bytes4, BigUint::from(4u32)),
// //     //         ]);
// //     //
// //     //         let data_serial = data_src
// //     //             .iter()
// //     //             .map(|(k, v)| (BigUint::from_bytes_be(k.as_slice()), v.clone()))
// //     //             .collect::<HashMap<_, _>>();
// //     //
// //     //         let key_len_bits = 256;
// //     //         let mut builder = CellBuilder::new();
// //     //         builder.store_dict(key_len_bits, val_writer_unsigned_min_size, data_serial)?;
// //     //
// //     //         let dict_cell = builder.build()?;
// //     //         let parsed = dict_cell
// //     //             .parser()
// //     //             .load_dict(key_len_bits, key_reader_256bit, val_reader_uint)?;
// //     //
// //     //         assert_eq!(data_src, parsed);
// //     //         Ok(())
// //     //     }
// //     //
// //     //     #[test]
// //     //     fn test_reader_uint() -> anyhow::Result<()> {
// //     //         let data = HashMap::from([
// //     //             (BigUint::from(0u32), BigUint::from(4u32)),
// //     //             (BigUint::from(1u32), BigUint::from(5u32)),
// //     //             (BigUint::from(2u32), BigUint::from(6u32)),
// //     //             (BigUint::from(64u32), BigUint::from(7u32)),
// //     //         ]);
// //     //         let key_len_bits = 8;
// //     //         let mut builder = CellBuilder::new();
// //     //         builder.store_dict(key_len_bits, val_writer_unsigned_min_size, data.clone())?;
// //     //         let dict_cell = builder.build()?;
// //     //         let parsed = dict_cell
// //     //             .parser()
// //     //             .load_dict(key_len_bits, key_reader_uint, val_reader_uint)?;
// //     //         assert_eq!(data, parsed);
// //     //         Ok(())
// //     //     }
// //     //
// //     //     #[test]
// //     //     fn test_reader_cell() -> anyhow::Result<()> {
// //     //         let data = HashMap::from([
// //     //             (
// //     //                 BigUint::from(0u32),
// //     //                 ArcCell::new(Cell::new(vec![0], 20, vec![], false)?),
// //     //             ),
// //     //             (
// //     //                 BigUint::from(1u32),
// //     //                 ArcCell::new(Cell::new(vec![1], 20, vec![], false)?),
// //     //             ),
// //     //             (
// //     //                 BigUint::from(2u32),
// //     //                 ArcCell::new(Cell::new(vec![2], 20, vec![], false)?),
// //     //             ),
// //     //             (
// //     //                 BigUint::from(6u32),
// //     //                 ArcCell::new(Cell::new(vec![6], 20, vec![], false)?),
// //     //             ),
// //     //         ]);
// //     //         let key_len_bits = 8;
// //     //         let mut builder = CellBuilder::new();
// //     //         builder.store_dict(key_len_bits, val_writer_ref_cell, data.clone())?;
// //     //         let dict_cell = builder.build()?;
// //     //         let mut parser = dict_cell.parser();
// //     //         let parsed = parser.load_dict(key_len_bits, key_reader_uint, val_reader_ref_cell)?;
// //     //         assert_eq!(data, parsed);
// //     //         Ok(())
// //     //     }
// //     //
// // }
