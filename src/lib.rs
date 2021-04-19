/*
- load/store instructions
- memmap instructions (elevate, deelevate)
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

const READ: u8  = 0b100;
const WRITE: u8 = 0b010;
const EXEC: u8  = 0b001;

#[derive(Debug)]
pub struct InvalidMemoryAccess;

impl std::fmt::Display for InvalidMemoryAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Invalid memory access")
    }
}

impl std::error::Error for InvalidMemoryAccess { }

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
    // General purpose integer registers
    // Program counter is x13
    // Stack base pointer is x14
    // Stack pointer is x15
    xs: [u32; 16],

    // General purpose floating point registers
    fs: [f32; 16],

    // Flags
    // IIIIIIII LLLZVCNP AFRMT
    //            111111 11112222 22222233
    // 01234567 89012345 67890123 45678901
    // IIIIIIII - Interrupt mask
    // LLL      - Last interrupt
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

static F_ZERO: u32 = 11;
static F_OVERFLOW: u32 = 12;
static F_CARRY: u32 = 13;
static F_NEGATIVE: u32 = 14;
static F_PARITY: u32 = 15;
static F_NAN: u32 = 16;
static F_INFINITE: u32 = 17;
static F_USER_RING: u32 = 18;
static F_MEMMAP_ENABLE: u32 = 19;
static F_TRAP: u32 = 20;

macro_rules! clear_flags {
    ($self: ident, $($flags: ident),*) => {
        $self.flags &= !($((1 << $flags))|*);
    }
}

impl<T> CPU<T>
where T: Address
{
    pub fn new(t: T) -> CPU<T> {
        CPU {
            xs: [0; 16],
            fs: [0.0; 16],
            flags: 0,
            addressing: t,
        }
    }

    fn check_memory(&self, addr: u32, permissions: u8) -> Result<u32, InvalidMemoryAccess> {
        // TODO: actually do memory mapping stuff
        Ok(addr)
    }

    fn set_flag(&mut self, flag: u32, val: bool) {
        self.flags |= (val as u32) << flag;
    }

    fn get_flag(&self, flag: u32) -> bool {
        self.flags & (1 << flag) != 0
    }

    fn set_carry(&mut self, val: bool) {
        clear_flags!(self, F_CARRY);
        self.set_flag(F_CARRY, val);
    }

    fn set_user_ring(&mut self, val: bool) {
        if !self.get_flag(F_USER_RING) {
            clear_flags!(self, F_USER_RING);
            self.set_flag(F_USER_RING, val);
        } else {
            todo!("interrupt on invalid access");
        }
    }

    fn set_memmap_enable(&mut self, val: bool) {
        if !self.get_flag(F_USER_RING) {
            clear_flags!(self, F_MEMMAP_ENABLE);
            self.set_flag(F_MEMMAP_ENABLE, val);
        } else {
            todo!("interrupt on invalid access");
        }
    }

    fn set_trap(&mut self, val: bool) {
        clear_flags!(self, F_TRAP);
        self.set_flag(F_TRAP, val);
    }

    fn load_lit_int(&mut self, x0: usize) -> Result<(), InvalidMemoryAccess> {
        let data = (self.exec()? as u32) | (self.exec()? as u32) << 1 | (self.exec()? as u32) << 2 | (self.exec()? as u32) << 3;
        self.xs[x0] = data;
        self.update_flags_int(data);
        Ok(())
    }

    fn load_lit_float(&mut self, f0: usize) -> Result<(), InvalidMemoryAccess> {
        let data = (self.exec()? as u32) | (self.exec()? as u32) << 1 | (self.exec()? as u32) << 2 | (self.exec()? as u32) << 3;
        let data = f32::from_bits(data);
        self.fs[f0] = data;
        self.update_flags_float(data);
        Ok(())
    }

    fn load_int(&mut self, x0: usize) -> Result<(), InvalidMemoryAccess> {
        let addr = (self.exec()? as u32) | (self.exec()? as u32) << 1 | (self.exec()? as u32) << 2 | (self.exec()? as u32) << 3;
        let data = (self.read(addr)? as u32) | (self.read(addr + 1)? as u32) << 1 | (self.read(addr + 2)? as u32) << 2 | (self.read(addr + 3)? as u32) << 3;
        self.xs[x0] = data;
        self.update_flags_int(data);
        Ok(())
    }

    fn load_float(&mut self, f0: usize) -> Result<(), InvalidMemoryAccess> {
        let addr = (self.exec()? as u32) | (self.exec()? as u32) << 1 | (self.exec()? as u32) << 2 | (self.exec()? as u32) << 3;
        let data = (self.read(addr)? as u32) | (self.read(addr + 1)? as u32) << 1 | (self.read(addr + 2)? as u32) << 2 | (self.read(addr + 3)? as u32) << 3;
        let data = f32::from_bits(data);
        self.fs[f0] = data;
        self.update_flags_float(data);
        Ok(())
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

    fn bsl(&mut self, x0: usize, x1: usize) {
        let res = if self.xs[x1] < 32 {
            (self.xs[x0] as u64) << self.xs[x1] as u64
        } else {
            0
        } | self.get_flag(F_CARRY) as u64;

        clear_flags!(self, F_ZERO, F_CARRY, F_NEGATIVE, F_PARITY);
        self.set_flag(F_ZERO, res as u32 == 0);
        self.set_flag(F_NEGATIVE, res & 0x80000000 != 0);
        if self.xs[x1] == 1 {
            self.set_flag(F_CARRY, res & 0x100000000 != 0);
        }
        self.set_flag(F_PARITY, res & 1 != 0);
        self.xs[x0] = res as u32;
    }

    fn bsr(&mut self, x0: usize, x1: usize) {
        let res = if self.xs[x1] < 32 {
            (self.xs[x0] as u64) >> self.xs[x1] as u64
        } else {
            0
        } | self.get_flag(F_CARRY) as u64;

        clear_flags!(self, F_ZERO, F_CARRY, F_NEGATIVE, F_PARITY);
        self.set_flag(F_ZERO, res as u32 == 0);
        self.set_flag(F_NEGATIVE, res & 0x80000000 != 0);
        if self.xs[x1] == 1 {
            self.set_flag(F_CARRY, self.xs[x0] & 1 != 0);
        }
        self.set_flag(F_PARITY, res & 1 != 0);
        self.xs[x0] = res as u32;
    }

    fn and(&mut self, x0: usize, x1: usize) {
        self.xs[x0] &= self.xs[x1];
        self.update_flags_int(self.xs[x0]);
    }

    fn or(&mut self, x0: usize, x1: usize) {
        self.xs[x0] |= self.xs[x1];
        self.update_flags_int(self.xs[x0]);
    }

    fn xor(&mut self, x0: usize, x1: usize) {
        self.xs[x0] ^= self.xs[x1];
        self.update_flags_int(self.xs[x0]);
    }

    fn move_int(&mut self, x0: usize, x1: usize) {
        self.xs[x0] = self.xs[x1];
        self.update_flags_int(self.xs[x0]);
    }

    fn move_float(&mut self, x0: usize, x1: usize) {
        self.fs[x0] = self.fs[x1];
        self.update_flags_float(self.fs[x0]);
    }

    fn move_int_float(&mut self, x0: usize, f1: usize) {
        self.xs[x0] = self.fs[f1] as u32;
        self.update_flags_int(self.xs[x0]);
    }

    fn move_float_int(&mut self, f0: usize, x1: usize) {
        self.fs[f0] = self.xs[x1] as f32;
        self.update_flags_float(self.fs[f0]);
    }

    fn transmute_int_float(&mut self, x0: usize, f1: usize) {
        self.xs[x0] = self.fs[f1].to_bits();
        self.update_flags_int(self.xs[x0]);
    }

    fn transmute_float_int(&mut self, f0: usize, x1: usize) {
        self.fs[f0] = f32::from_bits(self.xs[x1]);
        self.update_flags_float(self.fs[f0]);
    }

    fn load_indirect_int(&mut self, x0: usize, addr: usize) -> Result<(), InvalidMemoryAccess> {
        let addr = addr as u32;
        let data = (self.read(addr)? as u32) | (self.read(addr + 1)? as u32) << 1 | (self.read(addr + 2)? as u32) << 2 | (self.read(addr + 3)? as u32) << 3;
        self.xs[x0] = data;
        self.update_flags_int(data);
        Ok(())
    }

    fn load_indirect_float(&mut self, f0: usize, addr: usize) -> Result<(), InvalidMemoryAccess> {
        let addr = addr as u32;
        let data = (self.read(addr)? as u32) | (self.read(addr + 1)? as u32) << 1 | (self.read(addr + 2)? as u32) << 2 | (self.read(addr + 3)? as u32) << 3;
        let data = f32::from_bits(data);
        self.fs[f0] = data;
        self.update_flags_float(data);
        Ok(())
    }

    fn exec(&mut self) -> Result<u8, InvalidMemoryAccess> {
        self.check_memory(self.xs[13], EXEC)?;
        let res = self.addressing.read(self.xs[13]);
        self.xs[13] += 1;
        Ok(res)
    }

    fn read(&mut self, addr: u32) -> Result<u8, InvalidMemoryAccess> {
        self.check_memory(addr, READ)?;
        Ok(self.addressing.read(addr))
    }

    fn write(&mut self, addr: u32, data: u8) -> Result<(), InvalidMemoryAccess> {
        self.check_memory(addr, WRITE)?;
        self.addressing.write(addr, data);
        Ok(())
    }

    fn decode_instruction(&mut self, opcode: u8) -> Result<(), InvalidMemoryAccess> {
        match opcode & 0xc0 {
            // 0b00xxxxxx -> no arguments
            0x00 => {
                match opcode & 0x3f {
                    // Setting and clearing flags
                    0x00 => self.set_carry(false),
                    0x01 => self.set_carry(true),
                    0x02 => self.set_trap(false),
                    0x03 => self.set_trap(true),
                    0x04 => self.set_memmap_enable(false),
                    0x05 => self.set_memmap_enable(true),
                    0x06 => self.set_user_ring(false),
                    0x07 => self.set_user_ring(true),

                    _ => ()
                }
            }

            // 0b01xxyyyy data -> one register argument and 32 bit data
            0x40 => {
                let data = opcode as usize & 0x0f;
                match opcode & 0x30 {
                    // Load literal
                    0x00 => self.load_lit_int(data)?,
                    0x10 => self.load_lit_float(data)?,

                    // Load memory address
                    0x20 => self.load_int(data)?,
                    0x30 => self.load_float(data)?,

                    _ => unreachable!("nya :(")
                }
            }

            // 0b10xxxxxx 0byyyyzzzz -> two register arguments
            0x80 => {
                let data = self.exec()?;
                let (fst, snd) = (((data & 0xf0) >> 4) as usize, (data & 0x0f) as usize);

                match opcode & 0x3f {
                    // Integer arithmetic
                    0x00 => self.iadd(fst, snd),
                    0x01 => self.isub(fst, snd),
                    0x02 => self.imul(fst, snd),
                    0x03 => self.idiv(fst, snd),
                    0x04 => self.imod(fst, snd),

                    // Floating point arithmetic
                    0x05 => self.fadd(fst, snd),
                    0x06 => self.fsub(fst, snd),
                    0x07 => self.fmul(fst, snd),
                    0x08 => self.fdiv(fst, snd),

                    // Bitwise operations
                    0x09 => self.bsl(fst, snd),
                    0x0a => self.bsr(fst, snd),
                    0x0b => self.and(fst, snd),
                    0x0c => self.or(fst, snd),
                    0x0d => self.xor(fst, snd),

                    // Move and transmute operations
                    0x0e => self.move_int(fst, snd),
                    0x0f => self.move_float(fst, snd),
                    0x10 => self.move_int_float(fst, snd),
                    0x11 => self.move_float_int(fst, snd),
                    0x12 => self.transmute_int_float(fst, snd),
                    0x13 => self.transmute_float_int(fst, snd),

                    // Load operations
                    0x14 => self.load_indirect_int(fst, snd)?,
                    0x15 => self.load_indirect_float(fst, snd)?,

                    _ => ()
                }
            }

            // 0b11xxxxxx args -> miscellaneous arguments
            0xc0 => {
            }

            _ => unreachable!("nya :(")
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<(), InvalidMemoryAccess> {
        let opcode = self.exec()?;
        self.decode_instruction(opcode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_add() {
        let mut cpu = CPU::new(SimpleAddress::default());

        // Simple add
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

    #[test]
    fn cpu_bsl() {
        let mut cpu = CPU::new(SimpleAddress::default());

        // Simple bitshift
        cpu.xs[0] = 3;
        cpu.xs[1] = 2;
        cpu.bsl(0, 1);
        assert_eq!(cpu.xs[0], 12);
        assert!(!cpu.get_flag(F_CARRY));

        // Overflow
        cpu.xs[0] = 3;
        cpu.xs[1] = 32;
        cpu.bsl(0, 1);
        assert_eq!(cpu.xs[0], 0);
        assert!(!cpu.get_flag(F_CARRY));

        // Carry
        cpu.xs[0] = 0xffffffff;
        cpu.xs[1] = 1;
        cpu.bsl(0, 1);
        assert_eq!(cpu.xs[0], 0xfffffffe);
        assert!(cpu.get_flag(F_CARRY));
    }
}

