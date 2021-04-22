# cpuwu
Emulator for a custom 32 bit architecture with paging.

## Registers
The CPU has 16 32 bit integer registers, 16 32 bit floating point registers, 1 32 bit flag register, and 1 32 bit register that points to the structure that holds the paging tables. In total, there are 34 registers, all 32 bits (this is a 32 bit architecture after all). Some of the registers have special values, as indicated by the table below:
| Register | Type | Notes |
| :------: | ---- | ----- |
| `x0`     | int  |
| `x1`     | int  |
| `x2`     | int  |
| `x3`     | int  |
| `x4`     | int  |
| `x5`     | int  |
| `x6`     | int  |
| `x7`     | int  |
| `x8`     | int  |
| `x9`     | int  |
| `x10`    | int  |
| `x11`    | int  |
| `x12`    | int  |
| `x13`    | int  | Program counter    |
| `x14`    | int  | Stack base pointer |
| `x15`    | int  | Stack pointer      |
| `f0`     | float|
| `f1`     | float|
| `f2`     | float|
| `f3`     | float|
| `f4`     | float|
| `f5`     | float|
| `f6`     | float|
| `f7`     | float|
| `f8`     | float|
| `f9`     | float|
| `f10`    | float|
| `f11`    | float|
| `f12`    | float|
| `f13`    | float|
| `f14`    | float|
| `f15`    | float|
| `flags`  | int  | Contains flag information, see [flags](#flags) |
| `memmap` | int  | Contains the pointer to the page table         |

## Flags

## Opcodes
