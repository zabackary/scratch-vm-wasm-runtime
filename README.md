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

## Update about this project, in case you're interested:

I worked on this project for a little while in Summer 2023, and after getting a working prototype, it turned out that
having the overhead of a VM meant that there was not very much savings, and TurboWarp's compiled code was ~2x faster.
This could be because internally, the V8 JS engine (at least on Chrome) already compiles JS to machine code, and while
WASM is faster than JS because it can be optimized better even if it still needs JIT compilation, running a VM there and
running yet another layer of bytecode isn't very fast. If you want to optimize things to make it faster in the hopes
that it might pass TurboWarp, go ahead and submit a PR!

For future work (that I don't have time for, being a student and working on other projects), it might be possible to JIT
compile TurboWarp's Scratch AST (which is different from `scratch-vm`'s in that it's meant to be compiled) to actual
WebAssembly much like [v86](https://github.com/copy/v86#:~:text=Machine%20code%20is%20translated%20to%20WebAssembly%20modules%20at%20runtime%20in%20order%20to%20achieve%20decent%20performance.)
compiles x86 machine code to WASM bytecode. If you do this, please let me know! I'm curious to see whether it works.
