# cpuwu
Emulator for a custom 32 bit architecture with paging.

## Registers
The CPU has 16 32 bit integer registers, 16 32 bit floating point registers, 1 32 bit flag register, and 1 32 bit register that points to the structure that holds the paging tables. In total, there are 34 registers, all 32 bits (this is a 32 bit architecture after all). Some of the registers have special values, as indicated by the table below:
| Register   | Type | Notes
| :--------: | ---- | -----
| `x0`-`x12` | u32  | General purpose registers
| `x13`      | u32  | Program counter
| `x14`      | u32  | Stack base pointer
| `x15`      | u32  | Stack pointer
| `f0`-`f15` | f32  | General purpose registers
| `flags`    | u32  | Contains flag information, see [flags](#flags) for more details
| `mask`     | u8   | Contains the interrupt mask, see [interrupts](#interrupts) for more details
| `memmap`   | u32  | Contains the pointer to the page table

## Flags
The flags register is 32 bits, although almost half of the bits are currently unused. They are reserved for future expansion. The table below indicates the flags available:
```
                     MRFAN PCVZQLLL
10987654 32109876 54321098 76543210
33222222 22221111 111111
```
| Label      | Bit Range | Name              | Details
| ---------- | --------- | --------------    | -------
| `LLL`      | 0-2       | Last interrupt    | Identifier of the last interrupt called (0-7). See [interrupts](#interrupts) for more details.
| `Q`        | 3         | Enable interrupts | If set, interrupts are immediately requested; if unset, interrupts are queued.
| `Z`        | 4         | Zero              | Enabled if and only if the last operation resulted in a zero.
| `V`        | 5         | Overflow          | Enabled if and only if the last operation resulted in an overflow.
| `C`        | 6         | Carry             | Enabled if and only if the carry bit was set in the last operation. 
| `P`        | 7         | Parity            | Enabled if and only if the least significant bit of the last integer operation is enabled.
| `N`        | 8         | Negative          | Enabled if and only if the sign bit of the last integer operation is enabled.
| `A`        | 9         | NaN               | Enabled if and only if the last floating point operation resulted in a NaN.
| `F`        | 10        | Infinite          | Enabled if and only if the last floating point operation resulted in infinity.
| `R`        | 11        | User ring         | When enabled, the executed program has less permissions. See [rings](#rings) for more details.
| `M`        | 12        | Memory map        | When enabled, all operations to memory are passed through the paging table. See [paging](#paging) for more details.

## Rings
There are two protection rings: system and user. The ring the cpu is currently in is determined by the user ring flag. The system ring has unlimited access to hardware and can execute any instruction, including enabling and disabling paging, switching to the user ring, and modifying the contents of the flags directly. The user ring has limited access to hardware and can only be disabled via an interrupt.

## Paging
A page table is represented by two levels of tables. The first table is one kilobyte in size, and references other tables (not including itself) that are one kilobyte in size. Values that are zero in the first table are unused and can be allocated by the system as it wishes, whereas values in the second level of tables have their four most significant bits marked as indicated by the table below:
| Bit | Label
| --- | -----
| 0   | Used
| 1   | Readable
| 2   | Writable
| 3   | Executable
If an unavailable page is accessed, or a page without sufficient permissions is used, then the cpu will issue a page fault and an nonmaskable interrupt will occur.

## Interrupts
There are eight maskable interrupts. Interrupts are currently unimplemented so they do not have any documentation. :(

## Opcodes
A table of opcodes will be provided when the design is finalised.
