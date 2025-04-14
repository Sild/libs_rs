use std::collections::HashMap;

use super::label_type::LabelType;
use crate::cell::build_parse::parser::CellParser;
use crate::errors::TonLibError;
use crate::tlb::block::unary::Unary;
use crate::tlb::tlb_type::TLBType;
use num_bigint::BigUint;
use num_traits::One;

pub(crate) struct DictDataParser {
    key_bits_len: usize,
    cur_key_prefix: BigUint, // store leading 1 to determinate len properly
}

impl DictDataParser {
    pub(crate) fn new(key_len_bits: usize) -> DictDataParser {
        DictDataParser {
            key_bits_len: key_len_bits,
            cur_key_prefix: BigUint::one(),
        }
    }

    pub(crate) fn parse<V: TLBType>(&mut self, parser: &mut CellParser) -> Result<HashMap<BigUint, V>, TonLibError> {
        // reset state in case of reusing
        self.cur_key_prefix = BigUint::one();

        let mut result = HashMap::new();
        self.parse_impl(parser, &mut result)?;
        Ok(result)
    }

    fn parse_impl<V: TLBType>(
        &mut self,
        parser: &mut CellParser,
        dst: &mut HashMap<BigUint, V>,
    ) -> Result<(), TonLibError> {
        // will rollback prefix to original value at the end of the function
        let origin_key_prefix_len = self.cur_key_prefix.bits();

        let label_type = self.detect_label_type(parser)?;
        match label_type {
            LabelType::Same => {
                let prefix_val = parser.read_bit()?;
                let prefix_len_len = self.remain_suffix_bit_len();
                let prefix_len = parser.read_num::<usize>(prefix_len_len)?;
                if prefix_val {
                    self.cur_key_prefix += 1u32;
                    self.cur_key_prefix <<= prefix_len;
                    self.cur_key_prefix -= 1u32;
                } else {
                    self.cur_key_prefix <<= prefix_len;
                }
            }
            LabelType::Short => {
                let prefix_len = Unary::read(parser)?;
                if *prefix_len != 0 {
                    let val = parser.read_num::<BigUint>(*prefix_len)?;
                    self.cur_key_prefix <<= *prefix_len;
                    self.cur_key_prefix |= val;
                }
            }
            LabelType::Long => {
                let prefix_len_len = self.remain_suffix_bit_len();
                let prefix_len = parser.read_num::<u32>(prefix_len_len)?;
                if prefix_len_len != 0 {
                    let val = parser.read_num::<BigUint>(prefix_len)?;
                    self.cur_key_prefix <<= prefix_len;
                    self.cur_key_prefix |= val;
                }
            }
        }
        if self.cur_key_prefix.bits() as usize == (self.key_bits_len + 1) {
            let mut key = BigUint::one() << self.key_bits_len;
            key ^= &self.cur_key_prefix;
            dst.insert(key, V::read(parser)?);
        } else {
            let left_ref = parser.read_next_ref()?;
            self.cur_key_prefix <<= 1;
            self.parse_impl(&mut CellParser::new(left_ref), dst)?;

            let right_ref = parser.read_next_ref()?;
            self.cur_key_prefix += BigUint::one();
            self.parse_impl(&mut CellParser::new(right_ref), dst)?;
        }
        self.cur_key_prefix >>= self.cur_key_prefix.bits() - origin_key_prefix_len;
        Ok(())
    }

    fn detect_label_type(&self, parser: &mut CellParser) -> Result<LabelType, TonLibError> {
        let label = if parser.read_bit()? {
            if parser.read_bit()? {
                LabelType::Same
            } else {
                LabelType::Long
            }
        } else {
            LabelType::Short
        };
        Ok(label)
    }

    fn remain_suffix_bit_len(&self) -> u32 {
        // add 2 because cur_prefix contains leading bit
        let prefix_len_left = self.key_bits_len - self.cur_key_prefix.bits() as usize + 2;
        (prefix_len_left as f32).log2().ceil() as u32
    }
}
