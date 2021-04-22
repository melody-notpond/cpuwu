# cpuwu
Emulator for a custom 32 bit architecture with paging.

## Registers
The CPU has 16 32 bit integer registers, 16 32 bit floating point registers, 1 32 bit flag register, and 1 32 bit register that points to the structure that holds the paging tables. In total, there are 34 registers, all 32 bits (this is a 32 bit architecture after all). Some of the registers have special values, as indicated by the table below:
| Register   | Type | Notes |
| :--------: | ---- | ----- |
| `x0`-`x12` | int  | General purpose registers
| `x13`      | int  | Program counter
| `x14`      | int  | Stack base pointer
| `x15`      | int  | Stack pointer
| `f0`-`f15` | float| General purpose registers
| `flags`    | int  | Contains flag information, see [flags](#flags)
| `memmap`   | int  | Contains the pointer to the page table

## Flags
The flags register is 32 bits, although almost half of the bits are currently unused. They are reserved for future expansion, and also because `u24` is not a thing in Rust. The table below indicates the flags available:
```
IIIIIIII LLLZVCNP AFRM
           111111 11112222 22222233
01234567 89012345 67890123 45678901
```
| Label      | Bit Range | Name           | Details
| ---------- | --------- | -------------- | -------
| `IIIIIIII` | 0-7       | Interrupt mask | Determines which maskable interrupts can interrupt the program.
| `LLL`      | 8-10      | Last interrupt | Identifier of the last interrupt called (0-7).
| `Z`        | 11        | Zero           | Enabled if and only if the last operation resulted in a zero.
| `V`        | 12        | Overflow       | Enabled if and only if the last operation resulted in an overflow.
| `C`        | 13        | Carry          | Enabled if and only if the carry bit was set in the last operation. 
| `N`        | 14        | Negative       | Enabled if and only if the sign bit of the last integer operation is enabled.
| `P`        | 15        | Parity         | Enabled if and only if the least significant bit of the last integer operation is enabled.
| `A`        | 16        | NaN            | Enabled if and only if the last floating point operation resulted in a NaN.
| `F`        | 17        | Infinite       | Enabled if and only if the last floating point operation resulted in infinity.
| `R`        | 18        | User ring      | When enabled, the executed program has less permissions. See [rings](#rings) for more details.
| `M`        | 19        | Memory map     | When enabled, all operations to memory are passed through the paging table. See [paging](#paging) for more details.

## Rings
There are two protection rings: system and user. The ring the cpu is currently in is determined by the user ring flag. The system ring has unlimited access to hardware and can execute any instruction, including enabling and disabling paging, switching to the user ring, and modifying the contents of the flags directly. The user ring has limited access to hardware and can only be disabled via an interrupt.

## Paging

## Interrupts
There are eight maskable interrupts. Interrupts are currently unimplemented so they do not have any documentation. :(

## Opcodes
