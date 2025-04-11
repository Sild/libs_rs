use super::label_type::LabelType;
use super::leading_bit_utils::{add_leading_bit, all_bits_same, common_prefix_len, remove_leading_bit};
use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::ton_cell::TonCell;
use crate::errors::TonLibError;
use crate::errors::TonLibError::TLBDictWrongKeyLen;
use crate::tlb::primitives::unary_len::UnaryLen;
use crate::tlb::TLBType;
use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::mem::swap;

pub(crate) struct DictDataBuilder<V: TLBType> {
    keys_sorted: Vec<BigUint>, // contains 1 extra leading bit set to 1
    values_sorted: Vec<V>,
    key_bits_len_left: usize,
}

impl<V: TLBType> DictDataBuilder<V> {
    pub(crate) fn new(
        key_bits_len: usize,
        mut keys_sorted: Vec<BigUint>,
        values_sorted: Vec<V>,
    ) -> Result<Self, TonLibError> {
        // we support writing empty dict_data, but it's usually handled by 0 bit in parent cell
        prepare_keys(&mut keys_sorted, key_bits_len)?;
        let builder = DictDataBuilder {
            keys_sorted,
            values_sorted,
            key_bits_len_left: key_bits_len,
        };
        Ok(builder)
    }

    pub(crate) fn build(mut self) -> Result<TonCell, TonLibError> {
        let mut builder = CellBuilder::new();
        if self.keys_sorted.is_empty() {
            return builder.build();
        }
        let mut keys = vec![];
        swap(&mut self.keys_sorted, &mut keys);

        let keys = keys.into_iter().enumerate().collect();
        self.fill_cell(&mut builder, keys)?;
        builder.build()
    }

    // keys: Vec<(original_key_position, remaining_key_part)>
    fn fill_cell(&mut self, builder: &mut CellBuilder, keys: Vec<(usize, BigUint)>) -> Result<(), TonLibError> {
        if keys.len() == 1 {
            let (orig_key_pos, remaining_key) = &keys[0];
            return self.store_leaf(builder, *orig_key_pos, remaining_key);
        }

        // will restore it at the end
        let key_len_bits_left_original = self.key_bits_len_left;

        let key = &keys[0].1;
        let key_len = key.bits() as usize; // includes leading bit

        let common_prefix_len = common_prefix_len(key, &keys.last().unwrap().1);
        let label = {
            let ignored_suffix_len = key_len - common_prefix_len - 1;
            key >> ignored_suffix_len
        };
        self.store_label(builder, &label)?;

        let mut left_keys = Vec::with_capacity(keys.len() / 2);
        let mut right_keys = Vec::with_capacity(keys.len() / 2);

        let new_key_len = key_len - common_prefix_len - 1;
        let new_key_mask = (BigUint::one() << new_key_len) - 1u32;
        for (pos, key) in keys {
            let new_key = key & new_key_mask.clone();
            let is_right = new_key.bits() as usize == new_key_len;
            let new_key_internal = add_leading_bit(&new_key, new_key_len - 1);
            if is_right {
                right_keys.push((pos, new_key_internal));
            } else {
                left_keys.push((pos, new_key_internal));
            }
        }

        self.key_bits_len_left -= common_prefix_len + 1; // branch consumes 1 more bit
        let mut left_builder = CellBuilder::new();
        self.fill_cell(&mut left_builder, left_keys)?;
        builder.write_ref(left_builder.build()?.into_ref())?;

        let mut right_builder = CellBuilder::new();
        self.fill_cell(&mut right_builder, right_keys)?;
        builder.write_ref(right_builder.build()?.into_ref())?;

        self.key_bits_len_left = key_len_bits_left_original;
        Ok(())
    }

    fn store_leaf(
        &mut self,
        builder: &mut CellBuilder,
        orig_key_pos: usize,
        label: &BigUint,
    ) -> Result<(), TonLibError> {
        self.store_label(builder, label)?;
        self.values_sorted[orig_key_pos].write(builder)?;
        Ok(())
    }

    // expect label with leading one
    fn store_label(&self, builder: &mut CellBuilder, label: &BigUint) -> Result<(), TonLibError> {
        assert!(label.bits() > 0);
        if label.is_one() {
            // it's leading bit => label_type == short, len == 0 => store [false, false]
            builder.write_num(&0, 2)?;
            return Ok(());
        }
        let all_bits_same = all_bits_same(label);

        let label_len = label.bits() as u32 - 1;
        let label_len_len = (self.key_bits_len_left as f32 + 1.0).log2().ceil() as u32;
        let fair_label = remove_leading_bit(label);
        let same_label_len = if all_bits_same { 3 + label_len_len } else { u32::MAX };
        let short_label_len = 2 + label_len * 2;
        let long_label_len = 2 + label_len_len + label_len;

        let mut label_type = LabelType::Short;
        if long_label_len < short_label_len {
            label_type = LabelType::Long;
        }
        if same_label_len < short_label_len {
            label_type = LabelType::Same;
        }
        match label_type {
            LabelType::Same => {
                builder.write_bit(true)?;
                builder.write_bit(true)?;
                builder.write_bit(!fair_label.is_zero())?;
                builder.write_num(&label_len, label_len_len)?;
            }
            LabelType::Short => {
                builder.write_bit(false)?;
                let unary_len = UnaryLen(label_len);
                unary_len.write(builder)?;
                builder.write_num(&fair_label, label_len)?;
            }
            LabelType::Long => {
                builder.write_bit(true)?;
                builder.write_bit(false)?;
                builder.write_num(&label_len, label_len_len)?;
                builder.write_num(&fair_label, label_len)?;
            }
        }
        Ok(())
    }
}

fn prepare_keys(keys: &mut [BigUint], key_bits_len: usize) -> Result<(), TonLibError> {
    for key in keys {
        let received_len_bits = key.bits() as usize;
        if received_len_bits > key_bits_len {
            return Err(TLBDictWrongKeyLen {
                exp: key_bits_len,
                got: received_len_bits,
                key: key.clone(),
            });
        }

        // add leading bit to maintain proper bits length
        *key = add_leading_bit(key, key_bits_len);
    }
    Ok(())
}
