use std::{convert::TryInto, ops::IndexMut};

use chrono::prelude::*;

use crate::{
    instruction::{Instruction, InstructionType},
    scratch_value::ScratchValue,
};

/// Check an index against an array and see if it is out of bounds, but only
/// when the compiler option is enabled.
#[inline(always)]
fn bounds_check<T, I>(vec: &Vec<T>, idx: I) -> Result<(), &'static str>
where
    I: TryInto<usize>,
{
    #[cfg(feature = "safety_checks")]
    if idx.try_into().map_err(|_| "can't convert index")? >= vec.len() {
        return Err("index out of bounds");
    }
    return Ok(());
}

#[inline]
fn pop_stack(stack: &mut Vec<ScratchValue>) -> Result<ScratchValue, &'static str> {
    stack.pop().ok_or("nothing on the stack to pop")
}

fn scratch_find(list: &Vec<ScratchValue>, term: &str) -> usize {
    list.iter()
        // The call to `clone` may be a bottleneck -- is there any more efficient way?
        .position(|item| term.eq_ignore_ascii_case(&Into::<String>::into(item.clone())))
        .map_or(0, |a| a + 1)
}

/// Executes the instruction given by the argument, along with the stack,
/// constants, etc.
///
/// # Panics
///
/// Panics if the ID for a constant/variable/list is out of bounds. The compiler
/// should know better than that.
pub fn execute_instruction<F, G>(
    instruction: &Instruction,
    stack: &mut Vec<ScratchValue>,
    constants: &Vec<ScratchValue>,
    variables: &mut Vec<ScratchValue>,
    lists: &mut Vec<Vec<ScratchValue>>,
    jmp_consume_extra_arg: &mut F,
    return_control: &mut G,
) -> Result<(), &'static str>
where
    F: FnMut(isize) -> Option<u32>,
    G: FnMut(u32) -> (),
{
    match &instruction.name {
        InstructionType::Noop => Ok(()),
        InstructionType::ExtraArg => Err("Found ExtraArg where none was required"),
        InstructionType::LoadConst => {
            // Load a constant from the constants, falling back on an empty
            // string, and push it to the stack.
            #[cfg(feature = "safety_checks")]
            bounds_check(&constants, instruction.argument)?;
            stack.push(constants[instruction.argument as usize].clone());
            Ok(())
        }
        InstructionType::LoadConstInt => {
            stack.push(
                ScratchValue::Number(
                    unsafe { std::mem::transmute::<_, i32>(instruction.argument) }.into(),
                )
                .into(),
            );
            Ok(())
        }
        InstructionType::LoadConstFloat => {
            stack.push(ScratchValue::Number(unsafe {
                std::mem::transmute::<_, f32>(instruction.argument)
            } as f64));
            Ok(())
        }
        InstructionType::LoadConstBool => {
            stack.push(ScratchValue::Boolean(instruction.argument > 0));
            Ok(())
        }
        InstructionType::Load => {
            // Load a variable with the same schematics as above.
            #[cfg(feature = "safety_checks")]
            bounds_check(&variables, instruction.argument)?;
            stack.push(variables[instruction.argument as usize].clone());
            Ok(())
        }
        InstructionType::Store => {
            // Pop the top of the stack and store it
            #[cfg(feature = "safety_checks")]
            bounds_check(&variables, instruction.argument)?;
            variables[instruction.argument as usize] = pop_stack(stack)?;
            Ok(())
        }
        InstructionType::Jump => {
            // Jump by the argument, which is a i32, so we need to reinterpret
            // it as so using unsafe Rust.
            let offset = unsafe { std::mem::transmute::<_, i32>(instruction.argument) };
            jmp_consume_extra_arg(offset as isize);
            Ok(())
        }
        InstructionType::JumpIf => {
            // Jump by the argument if the top of the stack is truthful
            // Need same unsafe code as above, since the argument can be
            // negative
            if pop_stack(stack)?.into() {
                let offset = unsafe { std::mem::transmute::<_, i32>(instruction.argument) };
                jmp_consume_extra_arg(offset as isize);
            }
            Ok(())
        }
        InstructionType::AllocList => {
            // Get the list from the map
            #[cfg(feature = "safety_checks")]
            bounds_check(&lists, instruction.argument)?;
            let list = lists.index_mut(instruction.argument as usize);
            // Load the extra argument
            let additional_elements =
                jmp_consume_extra_arg(1).ok_or("ALLOC_LIST missing extra arg")?;
            // Check if allocation is too big, but only if safety_checks is
            // enabled
            #[cfg(feature = "safety_checks")]
            if additional_elements > 200_000 {
                return Err("allocation exceeds list limit");
            }
            // Attempt to allocate the vector, but if not possible, then
            // ignore the error
            let _ = list.try_reserve(additional_elements as usize);
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
        InstructionType::OpLt => {
            let lhs = pop_stack(stack)?;
            let rhs = pop_stack(stack)?;
            stack.push(ScratchValue::Boolean(
                Into::<f64>::into(lhs) < Into::<f64>::into(rhs),
            ));
            Ok(())
        }
        InstructionType::Reserved => todo!(),
        InstructionType::OpEq => {
            let lhs = pop_stack(stack)?;
            let rhs = pop_stack(stack)?;
            stack.push(ScratchValue::Boolean(
                Into::<f64>::into(lhs) == Into::<f64>::into(rhs),
            ));
            Ok(())
        }
        InstructionType::ListDel => {
            #[cfg(feature = "safety_checks")]
            bounds_check(&lists, instruction.argument)?;
            let list = lists.index_mut(instruction.argument as usize);
            let index = Into::<f64>::into(pop_stack(stack)?) as usize - 1;
            if index < list.len() {
                // If the index is out of bounds, no-op just like Scratch does
                list.remove(index);
            }
            Ok(())
        }
        InstructionType::ListIns => {
            #[cfg(feature = "safety_checks")]
            bounds_check(&lists, instruction.argument)?;
            let list = lists.index_mut(instruction.argument as usize);
            let element = pop_stack(stack)?;
            let index = Into::<f64>::into(pop_stack(stack)?) as usize - 1;
            if index <= list.len() {
                // If the index is out of bounds, no-op just like Scratch does
                list.insert(index, element);
            }
            Ok(())
        }
        InstructionType::ListDelAll => {
            #[cfg(feature = "safety_checks")]
            bounds_check(&lists, instruction.argument)?;
            let list = lists.index_mut(instruction.argument as usize);
            list.clear();
            // TODO: Deallocate vector? How?
            Ok(())
        }
        InstructionType::ListReplace => {
            #[cfg(feature = "safety_checks")]
            bounds_check(&lists, instruction.argument)?;
            let list = lists.index_mut(instruction.argument as usize);
            let element = pop_stack(stack)?;
            let index = Into::<f64>::into(pop_stack(stack)?) as usize - 1;
            if index < list.len() {
                // If the index is out of bounds, no-op just like Scratch does
                list[index] = element;
            }
            Ok(())
        }
        InstructionType::ListPush => {
            #[cfg(feature = "safety_checks")]
            bounds_check(&lists, instruction.argument)?;
            let list = lists.index_mut(instruction.argument as usize);
            let element = pop_stack(stack)?;
            list.push(element);
            Ok(())
        }
        InstructionType::ListLoad => {
            // no safety check because `get` does it for us
            let list = lists
                .get(instruction.argument as usize)
                .ok_or("failed to find list")?;
            let index = Into::<f64>::into(pop_stack(stack)?) as usize - 1;
            stack.push(if index < list.len() {
                list[index].clone()
            } else {
                ScratchValue::EMPTY
            });
            Ok(())
        }
        InstructionType::ListLen => {
            #[cfg(feature = "safety_checks")]
            bounds_check(&lists, instruction.argument)?;
            stack.push(ScratchValue::Number(
                lists[instruction.argument as usize].len() as f64,
            ));
            Ok(())
        }
        InstructionType::ListIFind => {
            // no safety check because `get` does it for us
            let list = lists
                .get(instruction.argument as usize)
                .ok_or("failed to find list")?;
            let term: String = pop_stack(stack)?.into();
            stack.push(ScratchValue::Number(scratch_find(&list, &term) as f64));
            Ok(())
        }
        InstructionType::ListIIncludes => {
            // no safety check because `get` does it for us
            let list = lists
                .get(instruction.argument as usize)
                .ok_or("failed to find list")?;
            let term: String = pop_stack(stack)?.into();
            stack.push(ScratchValue::Boolean(scratch_find(&list, &term) > 0));
            Ok(())
        }
        InstructionType::MonitorShowVar => todo!(),
        InstructionType::MonitorHideVar => todo!(),
        InstructionType::MonitorShowList => todo!(),
        InstructionType::MonitorHideList => todo!(),
        InstructionType::Return => {
            return_control(instruction.argument);
            Ok(())
        }
        InstructionType::OpMod => {
            let lhs = pop_stack(stack)?;
            let rhs = pop_stack(stack)?;
            stack.push(lhs % rhs);
            Ok(())
        }
        InstructionType::StringIndexChar => {
            let index = Into::<f64>::into(pop_stack(stack)?);
            let string = Into::<String>::into(pop_stack(stack)?);
            stack.push(
                if index % 1.0 == 0.0 && index > 0.0 && (index as usize) <= string.len() {
                    ScratchValue::String(string[(index as usize - 1)..(index as usize)].to_string())
                } else {
                    ScratchValue::EMPTY
                },
            );
            Ok(())
        }
        InstructionType::StringLen => {
            let string = Into::<String>::into(pop_stack(stack)?);
            stack.push(ScratchValue::Number(string.len() as f64));
            Ok(())
        }
        InstructionType::StringConcat => {
            let rhs = Into::<String>::into(pop_stack(stack)?);
            let mut lhs = Into::<String>::into(pop_stack(stack)?);
            lhs.push_str(&rhs);
            stack.push(ScratchValue::String(lhs));
            Ok(())
        }
        InstructionType::UnaryRound => {
            let op = Into::<f64>::into(pop_stack(stack)?);
            stack.push(ScratchValue::Number(op.round()));
            Ok(())
        }
        InstructionType::DataRand => {
            if !cfg!(target_arch = "wasm32") {
                return Err("cannot generate random numbers if not on WASM");
            }

            let max = pop_stack(stack)?;
            let min = pop_stack(stack)?;
            let fractional_part = instruction.argument > 0;
            let num_min = Into::<f64>::into(min);
            let num_max = Into::<f64>::into(max);
            let rand = js_sys::Math::random();
            stack.push(ScratchValue::Number(if fractional_part {
                (rand * (num_max - num_min)) + num_min
            } else {
                num_min + (rand * ((num_max + 1.0) - num_min)).floor()
            }));
            Ok(())
        }
        InstructionType::DataDate => {
            stack.push(ScratchValue::Number(Local::now().day() as f64));
            Ok(())
        }
        InstructionType::DataWeekday => {
            stack.push(ScratchValue::Number(
                Local::now().weekday().number_from_monday() as f64,
            ));
            Ok(())
        }
        InstructionType::DataDaysSince2000 => {
            stack.push(ScratchValue::Number(
                (Local::now().timestamp_millis() as f64 - 946684800000.0)
                    / (24.0 * 60.0 * 60.0 * 1000.0),
            ));
            Ok(())
        }
        InstructionType::DataHour => {
            stack.push(ScratchValue::Number(Local::now().hour() as f64));
            Ok(())
        }
        InstructionType::DataMinute => {
            stack.push(ScratchValue::Number(Local::now().minute() as f64));
            Ok(())
        }
        InstructionType::DataMonth => {
            stack.push(ScratchValue::Number(Local::now().month() as f64));
            Ok(())
        }
        InstructionType::DataSecond => {
            stack.push(ScratchValue::Number(Local::now().second() as f64));
            Ok(())
        }
        InstructionType::DataYear => {
            stack.push(ScratchValue::Number(Local::now().year() as f64));
            Ok(())
        }
        #[allow(unreachable_patterns)]
        _ => Err("found unknown instruction"),
    }
}
