# Bytecode schematics

## High-level explanation

If you're familiar with the CPython bytecode, this may be familiar.

Most instructions are made up of three parts: A two-byte instruction name, a
two-byte reserved padding (for alignment and extensibility), and a four-byte
argument. Extra arguments may be passed via an `EXTRA_ARG` instruction
immediately following the instruction requiring it.

Everything is little-endian[^1]. TODO: Just use the endianness of the host?

## Example bytecode

Scratch:

```
when flag clicked
set [my variable v] to (0)
delete all of [my list v]
repeat (10)
  change (my variable) by [1]
  add (my variable) to [my list v]
end
```

Bytecode:

| Type     | Name          | ID  | Value |
| -------- | ------------- | --- | ----- |
| constant | N/A           | `0` | `0`   |
| constant | N/A           | `1` | `10`  |
| constant | N/A           | `2` | `1`   |
| variable | `my variable` | `0` | `0`   |
| variable | `_$i1`        | `1` | `0`   |
| list     | `my list`     | `0` | ` `   |

```
00 LOAD_CONST 0
01 STORE 0
02 ALLOC_LIST 0
03 EXTRA_ARG 10
04 LOAD_CONST 0
05 STORE 1
06 LOAD 1
07 LOAD_CONST 1
08 OP_LT
09 JUMP_IF 10
0a LOAD 0
0b LOAD_CONST 2
0c OP_ADD
0d STORE 0
0e LOAD 0
0f LIST_PUSH 0
10 RETURN
```

## Questions for consideration

- Should there a be a `STORE_NOPOP`? Many `STORE` instructions are followed by a
  `LOAD` of the same variable.

## Instructions

| Name                 | Hex      | Description                                                                                                                                        |
| -------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `NOOP`               | `0x0000` | Does nothing. A no-op.                                                                                                                             |
| `EXTRA_ARG`          | `0x0001` | An extra argument to pass to the preceding instruction.                                                                                            |
| `LOAD_CONST`         | `0x0002` | Loads the constant specified in the argument to the top of the stack.                                                                              |
| `LOAD`               | `0x0003` | Loads the Scratch variable specified to the top of the stack.                                                                                      |
| `STORE`              | `0x0004` | Writes the value on the top of the stack to the specified slot[^2] and pops it.                                                                    |
| `JUMP`               | `0x0005` | Jumps by the offset specified by the argument[^5] (relative).                                                                                      |
| `JUMP_IF`            | `0x0006` | Jumps by the offset specified by the argument[^5] (relative), but only if the value at the top of the stack can be coerced to a boolean `true`     |
| `ALLOC_LIST`         | `0x0007` | Allocates the amount of elements specified by an `EXTRA_ARG` immediately following the instruction for the list specified by the argument.         |
| `OP_ADD`             | `0x0008` | Pops and adds the top two elements of the stack then puts the result.                                                                              |
| `OP_SUBTRACT`        | `0x0009` | Pops and subtracts the first element of the stack from the second of the stack then puts the result.                                               |
| `OP_MULTIPLY`        | `0x000a` | Pops and multiples the top two elements of the stack then puts the result.                                                                         |
| `OP_DIVIDE`          | `0x000b` | Pops and divides the second element of the stack by the first, then puts the result.                                                               |
| `OP_AND`             | `0x000c` | Implements `TOS = TOS1 && TOS2`, popping the top two elements of the stack.                                                                        |
| `OP_OR`              | `0x000d` | Implements `TOS = TOS1 \|\| TOS2`, popping the top two elements of the stack.                                                                      |
| `UNARY_NOT`          | `0x000e` | Implements `TOS = !TOS`, coercing the value to a boolean if necessary.                                                                             |
| `UNARY_ABS`          | `0x000f` | Pops and takes the absolute value of `TOS` and pushes it.                                                                                          |
| `UNARY_FLOOR`        | `0x0010` | Pops and takes the floor of `TOS` and pushes it.                                                                                                   |
| `UNARY_CEIL`         | `0x0011` | Pops and takes the ceiling of `TOS` and pushes it.                                                                                                 |
| `UNARY_SQRT`         | `0x0012` | Pops and takes the square root of `TOS` and pushes it.                                                                                             |
| `UNARY_SIN`          | `0x0013` | Pops and takes the sine[^3] of `TOS` and pushes it.                                                                                                |
| `UNARY_COS`          | `0x0014` | Pops and takes the cosine[^3] of `TOS` and pushes it.                                                                                              |
| `UNARY_TAN`          | `0x0015` | Pops and takes the tangent[^3] of `TOS` and pushes it.                                                                                             |
| `UNARY_ASIN`         | `0x0016` | Pops and takes the inverse sine[^3] of `TOS` and pushes it.                                                                                        |
| `UNARY_ACOS`         | `0x0017` | Pops and takes the inverse cosine[^3] of `TOS` and pushes it.                                                                                      |
| `UNARY_ATAN`         | `0x0018` | Pops and takes the inverse tangent[^3] of `TOS` and pushes it.                                                                                     |
| `UNARY_LN`           | `0x0019` | Pops and takes the natural logarithm of `TOS` and pushes it.                                                                                       |
| `UNARY_LOG`          | `0x001a` | Pops and takes the base 10 logarithm of `TOS` and pushes it.                                                                                       |
| `UNARY_EPOW`         | `0x001b` | Pops and raises _e_ to the power of `TOS` and pushes it.                                                                                           |
| `UNARY_10POW`        | `0x001c` | Pops and takes 10 to the power of `TOS` and pushes it.                                                                                             |
| `OP_LT`              | `0x001d` | Implements `TOS = TOS1 < TOS2` while popping the first two elements of the stack.                                                                  |
| _reserved_           | `0x001e` | _Note: should there be `OP_GT`? The compiler can just reverse the operands. Side effects?_                                                         |
| `OP_EQ`              | `0x001f` | Implements `TOS = TOS === TOS2` while popping the first two elements of the stack.                                                                 |
| `LIST_DEL`           | `0x0020` | Deletes, from the list identified by the argument, the `TOS`th element.                                                                            |
| `LIST_INS`           | `0x0021` | Inserts the `TOS` after the index `TOS2`[^4] in the list given by the argument. Pops both.                                                         |
| `LIST_DEL_ALL`       | `0x0022` | Deletes the entire list given by the argument. Implemented using `Vector::truncate`. TODO: Should this deallocate the vector?                      |
| `LIST_REPLACE`       | `0x0023` | Replaces the `TOS2`th[^4] item in the list given by the argument with `TOS` and pops both.                                                         |
| `LIST_PUSH`          | `0x0024` | Adds `TOS` to the list given by argument.                                                                                                          |
| `LIST_LOAD`          | `0x0025` | Loads the `TOS`th[^4] index from the list given by the argument onto the stack and pops.                                                           |
| `LIST_LEN`           | `0x0026` | Loads the current length of the list given by the argument onto the stack.                                                                         |
| `LIST_IFIND`         | `0x0027` | Finds the index[^4] containing the value `TOS` (popped) and pushes it to the stack, case-insensitively. List from argument.                        |
| `LIST_IINCLUDES`     | `0x0028` | Checks if the list given by argument contains `TOS` (popped) and pushes it to the stack, case-insensitively.                                       |
| `MONITOR_SHOWVAR`    | `0x0029` | Shows the variable given as the argument on the screen.                                                                                            |
| `MONITOR_HIDEVAR`    | `0x002a` | Hides the variable given as the argument on the screen.                                                                                            |
| `MONITOR_SHOWLIST`   | `0x002b` | Shows the list given as the argument on the screen.                                                                                                |
| `MONITOR_HIDELIST`   | `0x002c` | Hides the list given as the argument on the screen.                                                                                                |
| `RETURN`             | `0x002d` | Returns control to `scratch-gui` with the current instruction pointer and the TOS.                                                                 |
| `OP_MOD`             | `0x002e` | Pops and divides the second element of the stack by the first, then puts the remainder.                                                            |
| `STRING_INDEXCHAR`   | `0x002f` | Puts the character at position `TOS`[^4] in string `TOS2` (pops) at the top of the stack.                                                          |
| `STRING_LEN`         | `0x0030` | Gets the length (in characters) of the string at `TOS` (pops)                                                                                      |
| `STRING_CONCAT`      | `0x0031` | Concatenates `TOS1` and `TOS2` and pushes (pops both)                                                                                              |
| `UNARY_ROUND`        | `0x0032` | Rounds `TOS` to the nearest integer.                                                                                                               |
| `DATA_RAND`          | `0x0033` | Generates a random number between `TOS2` and `TOS1` (pops both and pushes). If the argument is positive, then generates a float instead of an int. |
| `DATA_DATE`          | `0x0034` | Gets the current day of the month (1-31)                                                                                                           |
| `DATA_WEEKDAY`       | `0x0035` | Gets the current day of the week (1-7)                                                                                                             |
| `DATA_DAYSSINCE2000` | `0x0036` | JavaScript `() => (Date.now() - 946684800000) / (24 * 60 * 60 * 1000)` (days since 2000 with fractional component)                                 |
| `DATA_HOUR`          | `0x0037` | Gets the hour.                                                                                                                                     |
| `DATA_MINUTE`        | `0x0038` | Gets the minute.                                                                                                                                   |
| `DATA_MONTH`         | `0x0039` | Gets the month.                                                                                                                                    |
| `DATA_SECOND`        | `0x003a` | Gets the second.                                                                                                                                   |
| `DATA_YEAR`          | `0x003b` | Gets the year.                                                                                                                                     |
| `LOAD_CONST_INT`     | `0x003c` | Loads the integer (an i32, not the standard u32) from the argument onto the stack.                                                                 |
| `LOAD_CONST_BOOL`    | `0x003d` | Loads the boolean (>1 = true, 0 = false) from the argument onto the stack.                                                                         |
| `LOAD_CONST_FLOAT`   | `0x003e` | Loads the float (an f32, not the standard u32) from the argument onto the stack.                                                                   |

[^2]:
    This should also mark the variable as changed so `scratch-gui` can update
    monitors. TODO: Decide whether the compiler should convert to radians.

[^3]: In degrees, sadly.
[^4]: Indexes in Scratch are one-based.
[^5]: The argument for `JUMP` and `JUMP_IF` is a `i32`, not the standard `u32`.

## Variable schematics

There are three stores passed to the runtime: constants, variables, and lists.
Constants never change throughout the lifetime of the program.

[^1]:
    If, like me, you often forget which is which, little-endian =
    least-significant first
