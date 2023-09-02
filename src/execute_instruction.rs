use std::collections::HashMap;

use wasm_bindgen::JsValue;
use web_sys::console;

use crate::{
    instruction::{Instruction, InstructionType},
    scratch_value::ScratchValue,
};

pub fn execute_instruction<F>(
    instruction: &Instruction,
    stack: &mut Vec<ScratchValue>,
    constant_map: &mut HashMap<u32, ScratchValue>,
    variable_map: &mut HashMap<u32, ScratchValue>,
    list_map: &mut HashMap<u32, Vec<ScratchValue>>,
    jmp_consume_extra_arg: &mut F,
) -> Result<(), &'static str>
where
    F: FnMut(Option<usize>) -> Option<u32>,
{
    match &instruction.name {
        InstructionType::Noop => Ok(()),
        InstructionType::ExtraArg => Err("Found ExtraArg where none was required"),
        InstructionType::LoadConst => {
            stack.push(
                constant_map
                    .get(&instruction.argument)
                    .unwrap_or_else(|| ScratchValue::EMPTY_REF)
                    .clone(),
            );
            Ok(())
        }
        InstructionType::Load => {
            stack.push(
                variable_map
                    .get(&instruction.argument)
                    .unwrap_or_else(|| ScratchValue::EMPTY_REF)
                    .clone(),
            );
            Ok(())
        }
        InstructionType::Store => {
            variable_map.insert(
                instruction.argument,
                stack.pop().unwrap_or_else(|| ScratchValue::EMPTY),
            );
            Ok(())
        }
        InstructionType::Jump => {
            jmp_consume_extra_arg(Some(instruction.argument as usize));
            Ok(())
        }
        InstructionType::JumpIf => {
            if stack.pop().unwrap_or_else(|| ScratchValue::EMPTY).into() {
                jmp_consume_extra_arg(Some(instruction.argument as usize));
            }
            Ok(())
        }
        InstructionType::AllocList => {
            let list = list_map.get_mut(&instruction.argument);
            let additional_elements =
                jmp_consume_extra_arg(None).ok_or("ALLOC_LIST missing extra arg")?;
            #[cfg(feature = "safety_checks")]
            if additional_elements > 200_000 {
                console::warn_1(&JsValue::from_str("can't allocate that much"));
                return Err("allocation exceeds list limit");
            }
            if let Some(list) = list {
                // Attempt to allocate the vector, but if not possible, then
                // ignore the error
                let _ = list.try_reserve(additional_elements as usize);
            }
            Ok(())
        }
        InstructionType::OpAdd => todo!(),
        InstructionType::OpSubtract => todo!(),
        InstructionType::OpMultiply => todo!(),
        InstructionType::OpDivide => todo!(),
        InstructionType::OpAnd => todo!(),
        InstructionType::OpOr => todo!(),
        InstructionType::UnaryNot => todo!(),
        InstructionType::UnaryAbs => todo!(),
        InstructionType::UnaryFloor => todo!(),
        InstructionType::UnaryCeil => todo!(),
        InstructionType::UnarySqrt => todo!(),
        InstructionType::UnarySin => todo!(),
        InstructionType::UnaryCos => todo!(),
        InstructionType::UnaryTan => todo!(),
        InstructionType::UnaryAsin => todo!(),
        InstructionType::UnaryAcos => todo!(),
        InstructionType::UnaryAtan => todo!(),
        InstructionType::UnaryLn => todo!(),
        InstructionType::UnaryLog => todo!(),
        InstructionType::UnaryEPow => todo!(),
        InstructionType::Unary10Pow => todo!(),
        InstructionType::OpLt => todo!(),
        InstructionType::Reserved => todo!(),
        InstructionType::OpEq => todo!(),
        InstructionType::ListDel => todo!(),
        InstructionType::ListIns => todo!(),
        InstructionType::ListDelAll => todo!(),
        InstructionType::ListReplace => todo!(),
        InstructionType::ListPush => todo!(),
        InstructionType::ListLoad => todo!(),
        InstructionType::ListLen => todo!(),
        InstructionType::ListIFind => todo!(),
        InstructionType::ListIIncludes => todo!(),
        InstructionType::MonitorShowVar => todo!(),
        InstructionType::MonitorHideVar => todo!(),
        InstructionType::MonitorShowList => todo!(),
        InstructionType::MonitorHideList => todo!(),
        InstructionType::Return => todo!(),
    }
}
