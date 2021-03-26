/*
lets say we have a 32 bit risc architecture that is like so:
- 1 stack register
- 1 program counter
- 16 32 bit integer registers
- 16 32 bit floating point registers
- flags (zero, overflow, carry, interrupt mask, floating point nan, floating point signalling nan)

- load/store instructions
- basic integer arithmetic instructions (add, sub, mul, div)
- basic floating point arithmetic instructions (add, sub, mul, div)
- 8 bit interrupt mask
- memmap instructions (elevate, deelevate)
- bitwise operations (bitshift, bitwise and/or/xor)
- jumps
- branching
- calling functions
- returning from functions
- returning from interrupts
- push/pop instructions

- system level, unlimited access to memory
- user level, limited access to memory

first 4 bits set aside for mmu:
- bit 0 - user readable
- bit 1 - user writable
- bit 2 - user executable
- bit 3 - available
*/

pub trait Address {
    fn read(&mut self, addr: u32) -> u8;

    fn write(&mut self, addr: u32, data: u8);
}

const SIMPLE_ADDRESS_SIZE: usize = 0x1000000;

pub struct SimpleAddress {
    memory: Vec<u8>,
}

impl Default for SimpleAddress {
    fn default() -> SimpleAddress {
        SimpleAddress {
            memory: vec![0; SIMPLE_ADDRESS_SIZE],
        }
    }
}

impl Address for SimpleAddress {
    fn read(&mut self, addr: u32) -> u8 {
        if addr < 0x1000000 {
            self.memory[addr as usize]
        } else {
            0
        }
    }

    fn write(&mut self, addr: u32, data: u8) {
        if addr < 0x1000000 {
            self.memory[addr as usize] = data;
        }
    }
}

pub struct CPU<T>
where T: Address
{
    // Registers
    // Program counter
    pc: u32,

    // Stack pointer
    sp: u32,

    // General purpose integer registers
    xs: [u32; 16],

    // General purpose floating point registers
    fs: [f32; 16],

    // Flags
    // IIIIIIII LLLDZVCN PAFRMT
    //            111111 11112222 22222233
    // 01234567 89012345 67890123 45678901
    // IIIIIIII - Interrupt mask
    // LLL      - Last interrupt
    // D        - Disable interrupt
    // Z        - Zero
    // V        - oVerflow
    // C        - Carry
    // N        - Negative
    // P        - Parity
    // A        - nAn
    // F        - inFinite
    // R        - user Ring
    // M        - Memory map enable
    // T        - Trap (for debuggers)
    flags: u32,

    addressing: T,
}

/*
have 00xxxxxx be instructions that dont take any arguments
have 01xxyyyy be instructions that take one register argument
have 10xxxxxx be instructions that take two register arguments in another byte
and 11xxxxxx are miscellaneous instructions that take word arguments
*/

static F_DISABLE_INTERRUPT: u32 = 11;
static F_ZERO: u32 = 12;
static F_OVERFLOW: u32 = 13;
static F_CARRY: u32 = 14;
static F_NEGATIVE: u32 = 15;
static F_PARITY: u32 = 16;
static F_NAN: u32 = 17;
static F_INFINITE: u32 = 18;
static F_USER_RING: u32 = 19;
static F_MEMMAP_ENABLE: u32 = 20;
static F_TRAP: u32 = 21;

macro_rules! clear_flags {
    ($self: ident, $($flags: ident),*) => {
        $self.flags &= !($((1 << $flags))|*);
    }
}

#[allow(clippy::unusual_byte_groupings)]
impl<T> CPU<T>
where T: Address
{
    pub fn new(t: T) -> CPU<T> {
        CPU {
            pc: 0,
            sp: 0,
            xs: [0; 16],
            fs: [0.0; 16],
            flags: 0,
            addressing: t,
        }
    }

    fn check_memory(&self) -> Option<u32> {
        None
    }

    fn set_flag(&mut self, flag: u32, val: bool) {
        self.flags |= (val as u32) << flag;
    }

    fn get_flag(&self, flag: u32) -> bool {
        self.flags & (1 << flag) != 0
    }

    fn iadd(&mut self, x0: usize, x1: usize) {
        let res = self.xs[x0] as u64 + self.xs[x1] as u64 + self.get_flag(F_CARRY) as u64;
        clear_flags!(self, F_ZERO, F_OVERFLOW, F_CARRY, F_NEGATIVE, F_PARITY);
        self.set_flag(F_ZERO, res as u32 == 0);
        self.set_flag(F_NEGATIVE, res & 0x80000000 != 0);
        self.set_flag(F_CARRY, res & 0x100000000 != 0);
        self.set_flag(F_OVERFLOW, self.xs[x0] & 0x80000000 == self.xs[x1] & 0x80000000 && self.xs[x0] & 0x80000000 != res as u32 & 0x80000000);
        self.set_flag(F_PARITY, res & 1 != 0);
        self.xs[x0] = res as u32;
    }

    fn isub(&mut self, x0: usize, x1: usize) {
        self.xs[x1] = !self.xs[x1];
        self.iadd(x0, x1);
        self.xs[x1] = !self.xs[x1];
    }

    fn update_flags_int(&mut self, x: u32) {
        clear_flags!(self, F_ZERO, F_NEGATIVE, F_PARITY);
        self.set_flag(F_ZERO, x == 0);
        self.set_flag(F_NEGATIVE, x & 0x80000000 != 0);
        self.set_flag(F_PARITY, x & 1 != 0);
    }

    fn imul(&mut self, x0: usize, x1: usize) {
        self.xs[x0] *= self.xs[x1];
        self.update_flags_int(self.xs[x0]);
    }

    fn idiv(&mut self, x0: usize, x1: usize) {
        self.xs[x0] /= self.xs[x1];
        self.update_flags_int(self.xs[x0]);
    }

    fn imod(&mut self, x0: usize, x1: usize) {
        self.xs[x0] %= self.xs[x1];
        self.update_flags_int(self.xs[x0]);
    }

    fn update_flags_float(&mut self, x: f32) {
        clear_flags!(self, F_ZERO, F_NEGATIVE, F_NAN, F_INFINITE);
        self.set_flag(F_ZERO, x == 0.0);
        self.set_flag(F_NEGATIVE, x.is_sign_negative());
        self.set_flag(F_NAN, x.is_nan());
        self.set_flag(F_INFINITE, x.is_infinite());
    }

    fn fadd(&mut self, f0: usize, f1: usize) {
        self.fs[f0] += self.fs[f1];
        self.update_flags_float(self.fs[f0]);
    }

    fn fsub(&mut self, f0: usize, f1: usize) {
        self.fs[f0] -= self.fs[f1];
        self.update_flags_float(self.fs[f0]);
    }

    fn fmul(&mut self, f0: usize, f1: usize) {
        self.fs[f0] *= self.fs[f1];
        self.update_flags_float(self.fs[f0]);
    }

    fn fdiv(&mut self, f0: usize, f1: usize) {
        self.fs[f0] /= self.fs[f1];
        self.update_flags_float(self.fs[f0]);
    }

    pub fn step(&mut self) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_add() {
        // Simple add
        let mut cpu = CPU::new(SimpleAddress::default());
        cpu.xs[0] = 5;
        cpu.xs[1] = 10;
        cpu.iadd(0, 1);
        assert_eq!(cpu.xs[0], 15);
        assert!(!cpu.get_flag(F_CARRY));
        assert!(!cpu.get_flag(F_OVERFLOW));
        assert!(!cpu.get_flag(F_NEGATIVE));

        // Overflow
        cpu.xs[0] = (1 << 31) - 1;
        cpu.xs[1] = 1;
        cpu.iadd(0, 1);
        assert_eq!(cpu.xs[0], 0x80000000);
        assert!(!cpu.get_flag(F_CARRY));
        assert!(cpu.get_flag(F_OVERFLOW));
        assert!(cpu.get_flag(F_NEGATIVE));

        // Carry
        cpu.xs[0] = 0xffffffff;
        cpu.xs[1] = 0;
        cpu.set_flag(F_CARRY, true);
        cpu.iadd(0, 1);
        assert_eq!(cpu.xs[0], 0);
        assert!(cpu.get_flag(F_CARRY));
        assert!(!cpu.get_flag(F_OVERFLOW));
        assert!(!cpu.get_flag(F_NEGATIVE));
    }
}

