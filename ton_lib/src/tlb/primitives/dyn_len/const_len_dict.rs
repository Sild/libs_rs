use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::errors::TonLibError;
use crate::tlb::primitives::dict_data::data_builder::DictDataBuilder;
use crate::tlb::tlb_type::TLBType;
use num_bigint::BigUint;
use std::collections::HashMap;
use std::hash::Hash;
// Implementations are highly inefficient.

impl<K, V> TLBType for HashMap<K, V>
where
    K: Into<BigUint> + Clone + Ord + Hash,
    V: TLBType,
{
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonLibError> {
        if parser.read_bit()? {
            todo!()
        } else {
            Ok(HashMap::new())
        }
    }

    fn write_definition(&self, dst: &mut CellBuilder) -> Result<(), TonLibError> {
        if self.is_empty() {
            dst.write_bit(false)?;
            return Ok(());
        }

        let mut keys: Vec<&K> = Vec::with_capacity(self.len());
        for key in self.keys() {
            keys.push(key);
        }
        keys.sort_unstable();
        let mut values_sorted = Vec::with_capacity(self.len());
        for key in &keys {
            let value = self.get(key).unwrap();
            values_sorted.push(value.clone());
        }
        let keys_sorted: Vec<BigUint> = keys.into_iter().map(|k| k.clone().into()).collect();
        let data_builder = DictDataBuilder::new(0, keys_sorted, values_sorted)?;
        let dict_data_cell = data_builder.build()?.into_ref();
        dst.write_bit(true)?;
        dst.write_ref(dict_data_cell)
    }
}
