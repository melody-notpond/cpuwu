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
The flags register is 32 bits, although almost half of the bits are currently unused. They are reserved for future expansion, and also because `u24` is not a thing in Rust. The table below indicates the flags available:
```
IIIIIIII LLLZVCNP AFRM
           111111 11112222 22222233
01234567 89012345 67890123 45678901
```
| Label    | Bit Range | Name           | Details
| -------- | --------- | -------------- | -------
| IIIIIIII | 0-7       | Interrupt mask | Determines which maskable interrupts can interrupt the program.
| LLL      | 8-10      | Last interrupt | Identifier of the last interrupt called (0-7).
| Z        | 11        | Zero           |
| V        | 12        | Overflow       |
| C        | 13        | Carry          |
| N        | 14        | Negative       | Enabled if and only if the sign bit of the last integer operation is enabled.
| P        | 15        | Parity         | Enabled if and only if the least significant bit of the last integer operation is enabled.
| A        | 11        | NaN            | Enabled if and only if the last floating point operation resulted in a NaN.
| F        | 11        | Infinite       | Enabled if and only if the last floating point operation resulted in infinity.
| R        | 11        | User ring      | When enabled, the executed program has less permissions. See [rings](#rings) for more details.
| M        | 11        | Memory map     | When enabled, all operations to memory are passed through the paging table. See [MMU](#mmu) for more details.

## MMU

## Interrupts
There are eight maskable interrupts. Interrupts are currently unimplemented so they do not have any documentation. :(

## Opcodes
