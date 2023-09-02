use std::collections::HashMap;

use wasm_bindgen::JsValue;
use web_sys::console;

use crate::{
    instruction::{Instruction, InstructionType},
    scratch_value::ScratchValue,
};

#[inline]
fn pop_stack(stack: &mut Vec<ScratchValue>) -> Result<ScratchValue, &'static str> {
    stack.pop().ok_or("nothing on the stack to pop")
}

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
        InstructionType::OpAdd => {
            let lhs = pop_stack(stack)?;
            let rhs = pop_stack(stack)?;
            stack.push(lhs + rhs);
            Ok(())
        }
        InstructionType::OpSubtract => {
            let rhs = pop_stack(stack)?;
            let lhs = pop_stack(stack)?;
            stack.push(lhs - rhs);
            Ok(())
        }
        InstructionType::OpMultiply => {
            let lhs = pop_stack(stack)?;
            let rhs = pop_stack(stack)?;
            stack.push(lhs * rhs);
            Ok(())
        }
        InstructionType::OpDivide => {
            let rhs = pop_stack(stack)?;
            let lhs = pop_stack(stack)?;
            stack.push(lhs / rhs);
            Ok(())
        }
        InstructionType::OpAnd => {
            let lhs = pop_stack(stack)?;
            let rhs = pop_stack(stack)?;
            stack.push(lhs & rhs);
            Ok(())
        }
        InstructionType::OpOr => {
            let lhs = pop_stack(stack)?;
            let rhs = pop_stack(stack)?;
            stack.push(lhs | rhs);
            Ok(())
        }
        InstructionType::UnaryNot => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Boolean(!Into::<bool>::into(op)));
            Ok(())
        }
        InstructionType::UnaryAbs => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(Into::<f64>::into(op).abs()));
            Ok(())
        }
        InstructionType::UnaryFloor => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(Into::<f64>::into(op).floor()));
            Ok(())
        }
        InstructionType::UnaryCeil => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(Into::<f64>::into(op).ceil()));
            Ok(())
        }
        InstructionType::UnarySqrt => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(Into::<f64>::into(op).sqrt()));
            Ok(())
        }
        InstructionType::UnarySin => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(
                Into::<f64>::into(op).to_radians().sin(),
            ));
            Ok(())
        }
        InstructionType::UnaryCos => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(
                Into::<f64>::into(op).to_radians().cos(),
            ));
            Ok(())
        }
        InstructionType::UnaryTan => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(
                Into::<f64>::into(op).to_radians().tan(),
            ));
            Ok(())
        }
        InstructionType::UnaryAsin => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(
                Into::<f64>::into(op).to_radians().asin(),
            ));
            Ok(())
        }
        InstructionType::UnaryAcos => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(
                Into::<f64>::into(op).to_radians().acos(),
            ));
            Ok(())
        }
        InstructionType::UnaryAtan => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(
                Into::<f64>::into(op).to_radians().atan(),
            ));
            Ok(())
        }
        InstructionType::UnaryLn => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(
                Into::<f64>::into(op).to_radians().ln(),
            ));
            Ok(())
        }
        InstructionType::UnaryLog => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(
                Into::<f64>::into(op).to_radians().log10(),
            ));
            Ok(())
        }
        InstructionType::UnaryEPow => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(
                std::f64::consts::E.powf(Into::<f64>::into(op).to_radians()),
            ));
            Ok(())
        }
        InstructionType::Unary10Pow => {
            let op = pop_stack(stack)?;
            stack.push(ScratchValue::Number(
                10.0_f64.powf(Into::<f64>::into(op).to_radians()),
            ));
            Ok(())
        }
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
