use wasm_bindgen::JsValue;
use web_sys::console;

use crate::execute_instruction::execute_instruction;
use crate::instruction::{Instruction, InstructionType};
use crate::scratch_value::ScratchValue;

pub fn run_instructions(
    program_counter: &mut usize,
    stack: &mut Vec<ScratchValue>,
    instructions: &[Instruction],
    constants: &Vec<ScratchValue>,
    variables: &mut Vec<ScratchValue>,
    lists: &mut Vec<Vec<ScratchValue>>,
) -> Result<Option<u32>, JsValue> {
    let mut early_return = None;
    while early_return.is_none() && *program_counter < instructions.len() {
        let instruction = &instructions[*program_counter];
        let result = execute_instruction(
            instruction,
            stack,
            constants,
            variables,
            lists,
            &mut |offset| {
                *program_counter = program_counter.saturating_add_signed(offset);
                #[cfg(feature = "safety_checks")]
                if *program_counter >= instructions.len() {
                    console::error_1(&JsValue::from_str("new counter too large"));
                    return None;
                }
                let instruction = &instructions[*program_counter];
                match instruction {
                    Instruction {
                        name: InstructionType::ExtraArg,
                        argument,
                        ..
                    } => Some(*argument),
                    _ => None,
                }
            },
            &mut |argument| {
                early_return = Some(argument);
            },
        );
        if let Err(res_err) = result {
            return Err(JsValue::from_str(&format!(
                "Instruction failed to execute (@{}): {} (stack = {:?})",
                *program_counter, res_err, stack
            )));
        }
        *program_counter += 1;
    }
    Ok(early_return)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::transmute_instructions;

    use super::*;

    #[test]
    fn test_runtime_noop() {
        let instructions = unsafe { transmute_instructions(&[0x0000000000000000u64]) };
        let mut program_counter = 0;
        let mut stack = vec![];
        run_instructions(
            &mut program_counter,
            &mut stack,
            instructions,
            &vec![],
            &mut vec![],
            &mut vec![],
        )
        .unwrap();
    }

    #[test]
    fn test_runtime_add() {
        let constants = vec![ScratchValue::Number(1.0)];
        let instructions = unsafe {
            transmute_instructions(&[
                0x0000000000000002u64, // LOAD_CONST 0
                0x0000000000000002u64, // LOAD_CONST 0
                0x0000000000000008u64, // OP_ADD
            ])
        };
        let mut program_counter = 0;
        let mut stack = vec![];
        run_instructions(
            &mut program_counter,
            &mut stack,
            instructions,
            &constants,
            &mut vec![],
            &mut vec![],
        )
        .unwrap();
        assert_eq!(
            stack,
            [ScratchValue::Number(2.0)],
            "stack isn't 2 after adding 1+1; stack={:?}",
            stack
        );
    }
}
