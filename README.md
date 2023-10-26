# `scratch-wm-wasm-runtime`

> Basically just a loop that runs bytecode instructions.

For an explainer, please see [this documentation](./docs/bytecode-schematics.md), which explains the format of the bytecode.

Code outline:

* `lib.rs` just binds the JS and the Rust code using `wasm-pack` and `wasm-bindgen`
* `runner.rs` just iterates over the list of instructions and runs them
* `execute_instruction.rs` executes individual instructions using a big `match` tree
* `instruction.rs` contains definitions for the instructions and the `struct` for their representation
* `scratch_value.rs` contains operations that act on Scratch-like polymorphic values.

The entire thing is a stack-based interpreter.
