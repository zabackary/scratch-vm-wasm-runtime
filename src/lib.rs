mod execute_instruction;
mod instruction;
mod runner;
mod scratch_value;
mod utils;

use std::convert::TryInto;

use instruction::Instruction;
use js_sys::{Array, Reflect};
use runner::run_instructions;
use scratch_value::ScratchValue;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
extern "C" {}

#[wasm_bindgen]
pub fn init() {
    set_panic_hook();
    console::log_1(&"Initialized scratch-vm-wasm-runtime".into());
}

#[wasm_bindgen(typescript_custom_section)]
const TS_DECLARATION: &'static str = r#"
export type VariableStore = Map<number, string | number | boolean>; 
"#;

unsafe fn transmute_instructions(bytecode: &[u64]) -> &[Instruction] {
    std::mem::transmute::<&[u64], &[Instruction]>(bytecode)
}

/// For lists, they are passed as strings with the null character as the list
/// item separator
/// This function is the "glue" binding the real logic in runner.rs to JS
#[wasm_bindgen]
pub fn run_sync(
    initial_program_counter: usize,
    initial_stack: Vec<JsValue>,
    bytecode: &[u64],
    constants_vec: Vec<JsValue>,
    variables_vec: Vec<JsValue>,
    lists_vec: Vec<JsValue>,
) -> Result<js_sys::Object, JsValue> {
    // Set the panic hook (remove if too slow? maybe just call init() from js?)
    // In theory it no-ops if already set
    set_panic_hook();
    // Load the instructions unsafely
    let instructions = unsafe { transmute_instructions(bytecode) };
    // Set up the stack
    let mut stack: Vec<ScratchValue> = Vec::with_capacity(initial_stack.len());
    for stack_item in initial_stack {
        stack.push(stack_item.try_into()?);
    }
    // Load the constants from the map
    let mut constants: Vec<ScratchValue> = Vec::with_capacity(constants_vec.len());
    for constant in constants_vec {
        constants.push(constant.try_into()?);
    }
    // Load the variables from the map
    let mut variables: Vec<ScratchValue> = Vec::with_capacity(variables_vec.len());
    for variable in variables_vec {
        variables.push(variable.try_into()?);
    }
    // Load the lists from the map
    let mut lists: Vec<Vec<ScratchValue>> = Vec::with_capacity(lists_vec.len());
    for list in lists_vec {
        let list_contents = list.as_string().ok_or("failed to parse list")?;
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
        lists.push(items);
    }

    let mut program_counter = initial_program_counter;
    let return_reason = run_instructions(
        &mut program_counter,
        &mut stack,
        instructions,
        &constants,
        &mut variables,
        &mut lists,
    )?;
    // Load the variable store into a Map for the response
    let response = js_sys::Object::new();
    Reflect::set(
        &response,
        &JsValue::from_str("variables"),
        &variables
            .into_iter()
            .map(|item| Into::<JsValue>::into(item))
            .collect::<Array>(),
    )?;
    Reflect::set(
        &response,
        &JsValue::from_str("lists"),
        &lists
            .into_iter()
            .map(|v| {
                Into::<JsValue>::into(
                    v.into_iter()
                        .map(|item| Into::<String>::into(item))
                        .collect::<Vec<_>>()
                        .join("\0"),
                )
            })
            .collect::<Array>(),
    )?;
    Reflect::set(
        &response,
        &JsValue::from_str("stack"),
        &stack
            .into_iter()
            .map(|item| Into::<JsValue>::into(item))
            .collect::<Array>(),
    )?;
    Reflect::set(
        &response,
        &JsValue::from_str("programCounter"),
        &JsValue::from_f64(program_counter as f64),
    )?;
    if let Some(return_argument) = return_reason {
        Reflect::set(
            &response,
            &JsValue::from_str("returnReason"),
            &JsValue::from_f64(return_argument as f64),
        )?;
    }
    Ok(response)
}
