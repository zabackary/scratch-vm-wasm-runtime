mod execute_instruction;
mod instruction;
mod scratch_value;
mod utils;

use std::{collections::HashMap, convert::TryInto};

use execute_instruction::execute_instruction;
use instruction::{Instruction, InstructionType};
use js_sys::Array;
use scratch_value::ScratchValue;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn init() {
    set_panic_hook();
    console::log_1(&"Initialized scratch-vm-wasm-runtime".into());
}

#[wasm_bindgen(typescript_custom_section)]
const TS_DECLARATION: &'static str = r#"
export type VariableStore = Map<string, string | number | boolean>; 
"#;

/// For lists, they are passed as strings with the null character as the list
/// item separator
#[wasm_bindgen]
pub fn run_sync(
    initial_program_counter: usize,
    initial_stack: Vec<JsValue>,
    bytecode: &[u64],
    constants: &js_sys::Map,
    variables: &js_sys::Map,
    lists: &js_sys::Map,
) -> Result<js_sys::Map, JsValue> {
    // Load the instructions unsafely
    let instructions = unsafe { std::mem::transmute::<&[u64], &[Instruction]>(bytecode) };
    // Set up the stack
    let mut stack: Vec<ScratchValue> = Vec::with_capacity(initial_stack.len());
    for stack_item in initial_stack {
        stack.push(stack_item.try_into()?);
    }
    // Load the constants from the map
    let mut constant_map = HashMap::<u32, ScratchValue>::new();
    for key in constants.keys() {
        constant_map.insert(
            key.clone()?.as_f64().ok_or("Failed to read key")? as u32,
            constants.get(&key?).try_into()?,
        );
    }
    // Load the variables from the map
    let mut variable_map = HashMap::<u32, ScratchValue>::new();
    for key in variables.keys() {
        variable_map.insert(
            key.clone()?.as_f64().ok_or("Failed to read key")? as u32,
            variables.get(&key?).try_into()?,
        );
    }
    // Load the lists from the map
    let mut list_map = HashMap::<u32, Vec<ScratchValue>>::new();
    for key in lists.keys() {
        let list_contents = lists
            .get(&key.clone()?)
            .as_string()
            .ok_or("Failed to parse list")?;
        // Create a new Vec for the list items, avoiding many allocations only
        // if the length of the concatenated contents is greater than 1000,
        // when the cost of allocating a lot may exceed the cost of counting.
        let mut items = if list_contents.len() > 1000 {
            Vec::with_capacity(list_contents.matches('\0').count() + 1)
        } else {
            Vec::new()
        };
        // Push each null-separated value to the Vec
        for list_item in list_contents.split("\0") {
            items.push(list_item.to_owned().into());
        }
        list_map.insert(key?.as_f64().ok_or("Failed to read key")? as u32, items);
    }
    // Execute the program
    let mut program_counter = initial_program_counter;
    let mut early_return = false;
    while !early_return && program_counter < instructions.len() {
        let instruction = &instructions[program_counter];
        execute_instruction(
            instruction,
            &mut stack,
            &mut constant_map,
            &mut variable_map,
            &mut list_map,
            &mut |new_program_counter| {
                program_counter = if let Some(pc) = new_program_counter {
                    pc
                } else {
                    program_counter + 1
                };
                #[cfg(safety_checks)]
                if program_counter > instructions.len() {
                    console::warn_1(JsValue::from_str("new counter too large"));
                    return None;
                }
                let instruction = &instructions[program_counter];
                match instruction {
                    Instruction {
                        name: InstructionType::ExtraArg,
                        argument,
                        ..
                    } => Some(*argument),
                    _ => None,
                }
            },
            &mut || {
                early_return = true;
            },
        )?;
        program_counter += 1;
    }
    // Load the variable store into a Map for the response
    let variables = js_sys::Map::new();
    for (k, v) in variable_map {
        variables.set(&JsValue::from_f64(k as f64), &v.into());
    }
    variables.set(
        &JsValue::from_str("_stack"),
        &stack
            .iter()
            .map(|item| Into::<JsValue>::into(item.clone()))
            .collect::<Array>(),
    );
    variables.set(
        &JsValue::from_str("_programCounter"),
        &JsValue::from_f64(program_counter as f64),
    );
    Ok(variables)
}
