# Bytecode schematics

## High-level explanation

If you're familiar with the CPython bytecode, this may be familiar.

Most instructions are made up of three parts: A two-byte instruction name, a
three-byte reserved padding (for alignment and extensibility), and a four-byte
argument. Extra arguments may be passed via an `EXTRA_ARG` instruction
immediately following the instruction requiring it.

Everything is little-endian[^1].

## Example bytecode

```

```

## Instructions

| Name          | Hex      | Description                                                                                                                                |
| ------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `NOOP`        | `0x0000` | Does nothing. A no-op.                                                                                                                     |
| `EXTRA_ARG`   | `0x0001` | An extra argument to pass to the preceding instruction.                                                                                    |
| `LOAD_CONST`  | `0x0002` | Loads the constant specified in the argument to the top of the stack.                                                                      |
| `LOAD`        | `0x0003` | Loads the Scratch variable specified to the top of the stack.                                                                              |
| `STORE`       | `0x0004` | Writes the value on the top of the stack to the specified slot[^2]                                                                         |
| `JUMP`        | `0x0005` | Jumps to the offset specified by the argument.                                                                                             |
| `JUMP_IF`     | `0x0006` | Jumps to the offset specified by the argument, but only if the value at the top of the stack can be coerced to a boolean `true`            |
| `ALLOC_LIST`  | `0x0007` | Allocates the amount of elements specified by an `EXTRA_ARG` immediately following the instruction for the list specified by the argument. |
| `OP_ADD`      | `0x0008` | Pops and adds the top two elements of the stack then puts the result.                                                                      |
| `OP_SUBTRACT` | `0x0009` | Pops and subtracts the first element of the stack from the second of the stack then puts the result.                                       |
| `OP_MULTIPLY` | `0x000a` | Pops and multiples the top two elements of the stack then puts the result.                                                                 |
| `OP_DIVIDE`   | `0x000b` | Pops and divides the second element of the stack by the first, then puts the result.                                                       |
| `OP_AND`      | `0x000c` | Implements `TOS = TOS1 && TOS2`, popping the top two elements of the stack.                                                                |
| `OP_OR`       | `0x000d` | Implements `TOS = TOS1 \|\| TOS2`, popping the top two elements of the stack.                                                              |
| `UNARY_NOT`   | `0x000e` | Implements `TOS = !TOS`, coercing the value to a boolean if necessary.                                                                     |
| `UNARY_ABS`   | `0x000f` | Pops and takes the absolute value of `TOS` and pushes it.                                                                                  |
| `UNARY_FLOOR` | `0x0010` | Pops and takes the floor of `TOS` and pushes it.                                                                                           |
| `UNARY_CEIL`  | `0x0011` | Pops and takes the ceiling of `TOS` and pushes it.                                                                                         |
| `UNARY_SQRT`  | `0x0012` | Pops and takes the square root of `TOS` and pushes it.                                                                                     |
| `UNARY_SIN`   | `0x0013` | Pops and takes the sine[^3] of `TOS` and pushes it.                                                                                        |
| `UNARY_COS`   | `0x0014` | Pops and takes the cosine[^3] of `TOS` and pushes it.                                                                                      |
| `UNARY_TAN`   | `0x0015` | Pops and takes the tangent[^3] of `TOS` and pushes it.                                                                                     |
| `UNARY_ASIN`  | `0x0016` | Pops and takes the inverse sine[^3] of `TOS` and pushes it.                                                                                |
| `UNARY_ACOS`  | `0x0017` | Pops and takes the inverse cosine[^3] of `TOS` and pushes it.                                                                              |
| `UNARY_ATAN`  | `0x0018` | Pops and takes the inverse tangent[^3] of `TOS` and pushes it.                                                                             |
| `UNARY_LN`    | `0x0019` | Pops and takes the natural logarithm of `TOS` and pushes it.                                                                               |
| `UNARY_LOG`   | `0x001a` | Pops and takes the base 10 logarithm of `TOS` and pushes it.                                                                               |
| `UNARY_EPOW`  | `0x001b` | Pops and raises _e_ to the power of `TOS` and pushes it.                                                                                   |
| `UNARY_10POW` | `0x001c` | Pops and takes 10 to the power of `TOS` and pushes it.                                                                                     |

[^2] This should also mark the variable as changed so `scratch-gui` can update
monitors.

[^3] In degrees, sadly.

## Variable schematics

There are three stores passed to the runtime: constants, variables, and lists.
Constants never change throughout the lifetime of the program.

[^1] If, like me, you often forget which is which, little-endian =
least-significant first
