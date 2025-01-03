use crate::instructions::{fields, IrOpcode};
use crate::prelude::*;
use crate::sb3::{Block, BlockArray, BlockInfo, BlockMap, BlockOpcode};
use fields::*;

impl BlockOpcode {
    fn from_block(block: &Block, blocks: &BlockMap) -> HQResult<IrOpcode> {
        match block {
            Block::Normal { block_info, .. } => BlockOpcode::from_normal_block(block_info, blocks),
            Block::Special(block_array) => BlockOpcode::from_special_block(block_array, blocks),
        }
    }

    fn from_normal_block(block_info: &BlockInfo, blocks: &BlockMap) -> HQResult<IrOpcode> {
        hq_todo!()
    }

    fn from_special_block(block_array: &BlockArray, blocks: &BlockMap) -> HQResult<IrOpcode> {
        Ok(match block_array {
            BlockArray::NumberOrAngle(ty, value) => match ty {
                4 | 8 => IrOpcode::math_number(MathNumberFields(*value)),
                5 => IrOpcode::math_positive_number(MathPositiveNumberFields(*value)),
                6 => IrOpcode::math_whole_number(MathWholeNumberFields(*value as i64)),
                7 => IrOpcode::math_integer(MathIntegerFields(*value as i64)),
                _ => hq_bad_proj!("bad project json (block array of type ({}, f64))", ty),
            },
            BlockArray::ColorOrString(ty, value) => match ty {
                4 | 8 => IrOpcode::math_number(MathNumberFields(
                    value.parse().map_err(|_| make_hq_bug!(""))?,
                )),
                5 => IrOpcode::math_positive_number(MathPositiveNumberFields(
                    value.parse().map_err(|_| make_hq_bug!(""))?,
                )),
                6 => IrOpcode::math_whole_number(MathWholeNumberFields(
                    value.parse().map_err(|_| make_hq_bug!(""))?,
                )),
                7 => IrOpcode::math_integer(MathIntegerFields(
                    value.parse().map_err(|_| make_hq_bug!(""))?,
                )),
                9 => hq_todo!(""),
                10 => hq_todo!(), /*IrOpcode::text {
                TEXT: value.to_string(),
                },*/
                _ => hq_bad_proj!("bad project json (block array of type ({}, string))", ty),
            },
            BlockArray::Broadcast(ty, _name, id)
            | BlockArray::VariableOrList(ty, _name, id, _, _) => match ty {
                /*12 => IrOpcode::data_variable {
                    VARIABLE: id.to_string(),
                    assume_type: None,
                },*/
                _ => hq_todo!(""),
            },
        })
    }
}
