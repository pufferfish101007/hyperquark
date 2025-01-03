use crate::ir::Type as IrType;
use crate::prelude::*;
use crate::wasm::StepFunc;
use wasm_encoder::{Instruction, ValType};

#[derive(Clone, Copy, Debug)]
pub struct Fields(pub i64);

pub fn wasm(
    _func: &StepFunc,
    _inputs: Rc<[IrType]>,
    fields: &Fields,
) -> HQResult<Vec<Instruction<'static>>> {
    Ok(vec![Instruction::I64Const(fields.0)])
}

pub fn acceptable_inputs() -> Rc<[IrType]> {
    Rc::new([])
}

pub fn output_type(_inputs: Rc<[IrType]>, &Fields(val): &Fields) -> HQResult<Option<IrType>> {
    Ok(Some(match val {
        0 => IrType::IntZero,
        pos if pos > 0 => IrType::IntPos,
        neg if neg < 0 => IrType::IntNeg,
        _ => unreachable!(),
    }))
}

crate::instructions_test! {tests;; super::Fields(0)}
