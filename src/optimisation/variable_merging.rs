use core::ops::Deref;
use core::cell::Ref;

use crate::instructions::{
    ControlIfElseFields, ControlLoopFields, DataSetvariabletoFields, DataTeevariableFields,
    DataVariableFields, HqYieldFields, IrOpcode, YieldMode,
};
use crate::ir::{IrProject, RcVar, Step};
use crate::optimisation::SSAToken;
use crate::prelude::*;

#[expect(clippy::mutable_key_type, reason = "hash depends only on immutable id")]
pub fn merge_vars_in_step<'a>(
    step: Ref<'a, Step>,
    live_variables: &mut BTreeMap<RcVar, Vec<&'a RefCell<RcVar>>>,
    dead_variables: &mut BTreeMap<RcVar, Vec<&'a RefCell<RcVar>>>,
    can_merge: bool,
) -> HQResult<()>
{
    let add_live_variable = |live_variables: &mut BTreeMap<RcVar, Vec<&'a RefCell<RcVar>>>,
                             dead_variables: &mut BTreeMap<RcVar, Vec<&'a RefCell<RcVar>>>,
                             var: &'a RefCell<RcVar>|
     -> HQResult<_> {
        live_variables
            .entry(var.try_borrow()?.clone())
            .or_default()
            .push(var);
        let removed = if can_merge
            && let Some((replaced, replacement_positions)) =
                dead_variables.iter().try_find(|(dead, _)| -> HQResult<_> {
                    Ok(*dead.possible_types() == *var.try_borrow()?.possible_types())
                })?
        {
            for &replacee in replacement_positions {
                *replacee.try_borrow_mut()? = var.try_borrow()?.clone();
            }
            crate::log("merged a variable");
            replaced.clone()
        } else {
            return Ok(())
        };
        dead_variables.remove(&removed);
        Ok(())
    };
    for block in step.opcodes().iter().rev() {
        #[expect(
            clippy::wildcard_enum_match_arm,
            reason = "too many variants to match individually"
        )]
        match block {
            IrOpcode::data_setvariableto(DataSetvariabletoFields {
                var,
                local_write,
                first_write,
            }) if *first_write.try_borrow()? => {
                if *local_write.try_borrow()? {
                    let mut vs = live_variables
                        .remove(&*var.try_borrow()?)
                        .unwrap_or_default();
                    vs.push(var);
                    dead_variables.insert(var.try_borrow()?.clone(), vs);
                }
            }
            IrOpcode::data_variable(DataVariableFields {
                var,
                local_read: local,
            })
            | IrOpcode::data_teevariable(DataTeevariableFields {
                var,
                local_read_write: local,
            }) => {
                if *local.try_borrow()? {
                    add_live_variable(live_variables, dead_variables, var)?;
                }
            }
            IrOpcode::data_setvariableto(DataSetvariabletoFields {
                var,
                local_write: local,
                first_write,
            }) if !*first_write.try_borrow()? => {
                if *local.try_borrow()? {
                    add_live_variable(live_variables, dead_variables, var)?;
                }
            }
            IrOpcode::hq_yield(HqYieldFields {
                mode: YieldMode::Inline(inline_step),
            }) => {
                let borrowed_inline_step = inline_step.try_borrow()?;
                merge_vars_in_step(
                    borrowed_inline_step,
                    live_variables,
                    dead_variables,
                    can_merge,
                )?;
            }
            IrOpcode::control_if_else(ControlIfElseFields {
                branch_if,
                branch_else,
            }) => {
                let borrowed_branch_if = branch_if.try_borrow()?;
                merge_vars_in_step(borrowed_branch_if, live_variables, dead_variables, false)?;
                let borrowed_branch_else = branch_else.try_borrow()?;
                merge_vars_in_step(
                    borrowed_branch_else,
                    live_variables,
                    dead_variables,
                    false,
                )?;
            }
            IrOpcode::control_loop(ControlLoopFields { .. }) => {
                crate::log("got control flow when merging variables");
                break;
            }
            _ => (),
        }
    }
    Ok(())
}

/// Merges variables that have the same type, only if their usage does not overlap.
///
/// This is important because we can sometimes produce far too many locals for the
/// browser to be able to compile the WASM module, and as Binaryen's coalesce-locals
/// pass is "non-linear in the number of locals", WASM optimisation can take an
/// unacceptably long time.
///
/// At the moment, this only merges variables that have exactly the same type, not just
/// the same basic type, to preserve the maximal amount of information.
///
/// This also bases its liveness analysis on information provided in `data_setvariableto`
/// instructions, so that it can run linearly-ish in the number of instructions. Therefore
/// it does need to be run after SSA.
///
/// TODO: make this configurable? Or find some other way to indicate the type of a variable
/// at a specific point in time.
pub fn merge_variables(proj: &Rc<IrProject>, _ssa_token: SSAToken) -> HQResult<()> {
    for step in proj.steps().borrow().iter() {
        merge_vars_in_step(
            step.try_borrow()?,
            &mut BTreeMap::default(),
            &mut BTreeMap::default(),
            true,
        )?;
    }

    Ok(())
}
