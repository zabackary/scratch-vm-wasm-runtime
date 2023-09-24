use std::collections::HashMap;

use wasm_bindgen::JsValue;

use crate::execute_instruction::execute_instruction;
use crate::instruction::{Instruction, InstructionType};
use crate::scratch_value::ScratchValue;

pub fn run_instructions(
    program_counter: &mut usize,
    stack: &mut Vec<ScratchValue>,
    instructions: &[Instruction],
    constants: &HashMap<u32, ScratchValue>,
    variables: &mut HashMap<u32, ScratchValue>,
    lists: &mut HashMap<u32, Vec<ScratchValue>>,
) -> Result<Option<u32>, JsValue> {
    let mut early_return = None;
    while early_return.is_none() && *program_counter < instructions.len() {
        let instruction = &instructions[*program_counter];
        execute_instruction(
            instruction,
            stack,
            constants,
            variables,
            lists,
            &mut |offset| {
                *program_counter = *program_counter + offset;
                #[cfg(safety_checks)]
                if program_counter > instructions.len() {
                    console::warn_1(JsValue::from_str("new counter too large"));
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
        )?;
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
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
        )
        .unwrap();
    }

    #[test]
    fn test_runtime_add() {
        let mut constants = HashMap::new();
        constants.insert(0, ScratchValue::Number(1.0));
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
            &mut HashMap::new(),
            &mut HashMap::new(),
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
