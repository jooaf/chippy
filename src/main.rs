use std::fs;

pub struct CPU {
    pub pc: usize,
    pub v0: u8,
    pub v1: u8,
    pub v2: u8,
    pub v3: u8,
    pub v4: u8,
    pub v5: u8,
    pub v6: u8,
    pub v7: u8,
    pub v8: u8,
    pub v9: u8,
    pub va: u8,
    pub vb: u8,
    pub vc: u8,
    pub vd: u8,
    pub ve: u8,
    pub vf: u8,
    // used for carry
    pub vi: u16,
    // used for memory addr
    pub stack: [u16; 16],
    pub sp: u8,
    pub memory: [u8; 0x1000],
    pub error: u8,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            pc: 0x200,
            v0: 0,
            v1: 0,
            v2: 0,
            v3: 0,
            v4: 0,
            v5: 0,
            v6: 0,
            v7: 0,
            v8: 0,
            v9: 0,
            va: 0,
            vb: 0,
            vc: 0,
            vd: 0,
            ve: 0,
            vf: 0,
            vi: 0,
            sp: 0,
            stack: [0; 16],
            memory: [0; 0x1000],
            error: 0,
        }
    }

    pub fn load_program(&mut self, rom: &[u8]) {
        self.memory[0x200..0x200 + rom.len()].copy_from_slice(rom);
    }

    pub fn get_instruction(&self) -> u16 {
        let high = self.memory[self.pc] as u16;
        let low = self.memory[self.pc + 1] as u16;
        (high << 8) | low
    }

    fn get_register_value(&self, reg: u8) -> u8 {
        match reg {
            0x0 => self.v0,
            0x1 => self.v1,
            0x2 => self.v2,
            0x3 => self.v3,
            0x4 => self.v4,
            0x5 => self.v5,
            0x6 => self.v6,
            0x7 => self.v7,
            0x8 => self.v8,
            0x9 => self.v9,
            0xa => self.va,
            0xb => self.vb,
            0xc => self.vc,
            0xd => self.vd,
            0xe => self.ve,
            0xf => self.vf,
            _ => self.error,
        }
    }

    fn set_register_value(&mut self, reg: u8, value: u8) {
        match reg {
            0x0 => self.v0 = value,
            0x1 => self.v1 = value,
            0x2 => self.v2 = value,
            0x3 => self.v3 = value,
            0x4 => self.v4 = value,
            0x5 => self.v5 = value,
            0x6 => self.v6 = value,
            0x7 => self.v7 = value,
            0x8 => self.v8 = value,
            0x9 => self.v9 = value,
            0xa => self.va = value,
            0xb => self.vb = value,
            0xc => self.vc = value,
            0xd => self.vd = value,
            0xe => self.ve = value,
            0xf => self.vf = value,
            _ => {}
        }
        self.pc += 2;
    }

    // 00E0 - clear display
    fn clear_00e0(&mut self) {
        todo!()
    }

    // 00EE - return from subroutine
    fn ret_00ee(&mut self) {
        if self.sp == 0 {
            // empty stack, pc unchanged (matching Gleam behaviour)
        } else {
            self.sp -= 1;
            self.pc = self.stack[self.sp as usize] as usize;
        }
    }

    // 1NNN - jump to address NNN
    fn jump_1nnn(&mut self, opcode: u16) {
        self.pc = (opcode & 0x0FFF) as usize;
    }

    // 2NNN - call subroutine at NNN
    // TODO: currently stores pc of the call instruction, not pc+2; ret returns to call site
    fn call_2nnn(&mut self, opcode: u16) {
        let addr = (opcode & 0x0FFF) as usize;
        self.stack[self.sp as usize] = self.pc as u16;
        self.sp += 1;
        self.pc = addr;
    }

    // 3XKK - skip next instruction if Vx == kk
    // TODO: on False, pc is not advanced (same instruction re-executes unless main loop handles it)
    fn skip_next_eq_3xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let kk = (opcode & 0x00FF) as u8;
        if self.get_register_value(x) == kk {
            self.pc += 2;
        }
    }

    // 4XKK - skip next instruction if Vx != kk
    // TODO: on False, pc is not advanced
    fn skip_next_neq_4xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let kk = (opcode & 0x00FF) as u8;
        if self.get_register_value(x) != kk {
            self.pc += 2;
        }
    }

    // 5XY0 - skip next instruction if Vx == Vy
    // TODO: on False, pc is not advanced
    fn skip_next_5xy0(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        if self.get_register_value(nibbles.x) == self.get_register_value(nibbles.y) {
            self.pc += 2;
        }
    }

    // 6XKK - set Vx = kk
    fn load_6xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let kk = (opcode & 0x00FF) as u8;
        self.set_register_value(x, kk);
    }

    // 7XKK - set Vx = Vx + kk
    fn add_7xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let kk = (opcode & 0x00FF) as u8;
        let val = self.get_register_value(x).wrapping_add(kk);
        self.set_register_value(x, val);
    }

    // 8XY0 - set Vx = Vy
    fn load_8xy0(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let vy = self.get_register_value(nibbles.y);
        self.set_register_value(nibbles.x, vy);
    }

    // 8XY1 - set Vx = Vx OR Vy
    fn or_8xy1(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let result = self.get_register_value(nibbles.x) | self.get_register_value(nibbles.y);
        self.set_register_value(nibbles.x, result);
    }

    // 8XY2 - set Vx = Vx AND Vy
    fn and_8xy2(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let result = self.get_register_value(nibbles.x) & self.get_register_value(nibbles.y);
        self.set_register_value(nibbles.x, result);
    }

    // 8XY3 - set Vx = Vx XOR Vy
    fn xor_8xy3(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let result = self.get_register_value(nibbles.x) ^ self.get_register_value(nibbles.y);
        self.set_register_value(nibbles.x, result);
    }

    // 8XY4 - set Vx = Vx + Vy, VF = carry
    fn add_8xy4(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let vx = self.get_register_value(nibbles.x);
        let vy = self.get_register_value(nibbles.y);
        let (result, carry) = vx.overflowing_add(vy);
        self.set_register_value(nibbles.x, result);
        self.vf = if carry { 1 } else { 0 };
    }

    // 8XY5 - set Vx = Vx - Vy, VF = NOT borrow
    // TODO: borrow is based on register index (x > y), not register values (Vx > Vy)
    fn sub_8xy5(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let vx = self.get_register_value(nibbles.x);
        let vy = self.get_register_value(nibbles.y);
        let borrow = nibbles.x > nibbles.y;
        let result = if borrow { vx.wrapping_sub(vy) } else { 0 };
        self.set_register_value(nibbles.x, result);
        self.vf = if borrow { 1 } else { 0 };
    }

    // 8XY6 - set Vx = Vx >> 1, VF = LSB before shift
    fn shr_8xy6(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let vx = self.get_register_value(nibbles.x);
        let lsb = vx & 0x01;
        self.set_register_value(nibbles.x, vx >> 1);
        self.vf = lsb;
    }

    // 8XY7 - set Vx = Vy - Vx, VF = NOT borrow
    // TODO: borrow is based on register index (x > y), not register values (Vx > Vy)
    fn subn_8xy7(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let vx = self.get_register_value(nibbles.x);
        let vy = self.get_register_value(nibbles.y);
        let borrow = nibbles.x > nibbles.y;
        let result = if borrow { vy.wrapping_sub(vx) } else { 0 };
        self.set_register_value(nibbles.x, result);
        self.vf = if borrow { 1 } else { 0 };
    }

    // 8XYE - set Vx = Vx << 1, VF = MSB before shift
    // TODO: MSB check uses register index bit instead of register value MSB (always 0)
    fn shl_8xye(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let vx = self.get_register_value(nibbles.x);
        let msb = (nibbles.x & 0x10) >> 4;
        self.set_register_value(nibbles.x, vx << 1);
        self.vf = msb;
    }

    // 9XY0 - skip next instruction if Vx != Vy
    // TODO: on False, pc is not advanced
    fn skip_next_9xy0(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        if self.get_register_value(nibbles.x) != self.get_register_value(nibbles.y) {
            self.pc += 2;
        }
    }

    // ANNN - set I = NNN
    fn set_annn(&mut self, opcode: u16) {
        self.vi = opcode & 0x0FFF;
        self.pc += 2;
    }

    // BNNN - jump to NNN + V0
    fn set_bnnn(&mut self, opcode: u16) {
        let addr = (opcode & 0x0FFF) as usize;
        self.pc = addr + self.v0 as usize;
    }

    // CXKK - set Vx = random byte AND kk
    fn rnd_cxkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let kk = (opcode & 0x00FF) as u8;
        let rand_byte: u8 = rand::random();
        self.set_register_value(x, rand_byte & kk);
    }

    pub fn opcode_mapper(&mut self) {
        let opcode = self.get_instruction();
        let head = ((opcode & 0xF000) >> 12) as u8;
        let tail = (opcode & 0x000F) as u8;

        match opcode {
            0x00E0 => self.clear_00e0(),
            0x00EE => self.ret_00ee(),
            _ => match head {
                0x1 => self.jump_1nnn(opcode),
                0x2 => self.call_2nnn(opcode),
                0x3 => self.skip_next_eq_3xkk(opcode),
                0x4 => self.skip_next_neq_4xkk(opcode),
                0x5 => self.skip_next_5xy0(opcode),
                0x6 => self.load_6xkk(opcode),
                0x7 => self.add_7xkk(opcode),
                0x8 => match tail {
                    0x0 => self.load_8xy0(opcode),
                    0x1 => self.or_8xy1(opcode),
                    0x2 => self.and_8xy2(opcode),
                    0x3 => self.xor_8xy3(opcode),
                    0x4 => self.add_8xy4(opcode),
                    0x5 => self.sub_8xy5(opcode),
                    0x6 => self.shr_8xy6(opcode),
                    0x7 => self.subn_8xy7(opcode),
                    0xe => self.shl_8xye(opcode),
                    _ => panic!("Unknown 8XY opcode: {:#06x}", opcode),
                },
                0x9 => self.skip_next_9xy0(opcode),
                0xa => self.set_annn(opcode),
                0xb => self.set_bnnn(opcode),
                0xc => self.rnd_cxkk(opcode),
                _ => panic!("Opcode not implemented: {:#06x}", opcode),
            },
        }
    }
}

pub struct ThreeNibbles {
    pub x: u8,
    pub y: u8,
    pub lowest: u8,
}

impl ThreeNibbles {
    pub fn new(opcode: u16) -> Self {
        let x = ((0x0F00 & opcode) >> 8) as u8;
        let y = ((0x00F0 & opcode) >> 4) as u8;
        let lowest = (0x000F & opcode) as u8;
        Self { x, y, lowest }
    }
}

fn main() {
    println!("Hello, world!");
    let rom = fs::read("dummy.ch8").expect("Was expecting to read the file");
    let mut chip8 = CPU::new();
    chip8.load_program(&rom);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cpu() -> CPU {
        CPU::new()
    }

    fn write_opcode(cpu: &mut CPU, opcode: u16) {
        cpu.memory[cpu.pc] = (opcode >> 8) as u8;
        cpu.memory[cpu.pc + 1] = (opcode & 0x00FF) as u8;
    }

    // --- ret_00ee ---

    #[test]
    fn test_ret_00ee_returns_to_stack_addr() {
        let mut cpu = make_cpu();
        cpu.stack[0] = 0x300;
        cpu.sp = 1;
        write_opcode(&mut cpu, 0x00EE);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, 0x300);
        assert_eq!(cpu.sp, 0);
    }

    #[test]
    fn test_ret_00ee_empty_stack_pc_unchanged() {
        let mut cpu = make_cpu();
        cpu.sp = 0;
        let original_pc = cpu.pc;
        write_opcode(&mut cpu, 0x00EE);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, original_pc);
    }

    // --- jump_1nnn ---

    #[test]
    fn test_jump_1nnn() {
        let mut cpu = make_cpu();
        write_opcode(&mut cpu, 0x1ABC);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, 0xABC);
    }

    // --- call_2nnn ---

    #[test]
    fn test_call_2nnn() {
        let mut cpu = make_cpu();
        let original_pc = cpu.pc;
        write_opcode(&mut cpu, 0x2345);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, 0x345);
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack[0], original_pc as u16);
    }

    // --- skip_next_eq_3xkk ---

    #[test]
    fn test_skip_next_eq_3xkk_equal_skips() {
        let mut cpu = make_cpu();
        cpu.v1 = 0x42;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0x3142); // skip if V1 == 0x42
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before + 2);
    }

    #[test]
    fn test_skip_next_eq_3xkk_not_equal_no_skip() {
        let mut cpu = make_cpu();
        cpu.v1 = 0x10;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0x3142); // skip if V1 == 0x42
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before); // pc not advanced
    }

    // --- skip_next_neq_4xkk ---

    #[test]
    fn test_skip_next_neq_4xkk_not_equal_skips() {
        let mut cpu = make_cpu();
        cpu.v2 = 0x10;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0x4242); // skip if V2 != 0x42
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before + 2);
    }

    #[test]
    fn test_skip_next_neq_4xkk_equal_no_skip() {
        let mut cpu = make_cpu();
        cpu.v2 = 0x42;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0x4242); // skip if V2 != 0x42
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before);
    }

    // --- skip_next_5xy0 ---

    #[test]
    fn test_skip_next_5xy0_equal_skips() {
        let mut cpu = make_cpu();
        cpu.v1 = 5;
        cpu.v2 = 5;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0x5120); // skip if V1 == V2
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before + 2);
    }

    #[test]
    fn test_skip_next_5xy0_not_equal_no_skip() {
        let mut cpu = make_cpu();
        cpu.v1 = 5;
        cpu.v2 = 6;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0x5120);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before);
    }

    // --- load_6xkk ---

    #[test]
    fn test_load_6xkk() {
        let mut cpu = make_cpu();
        write_opcode(&mut cpu, 0x63FF); // V3 = 0xFF
        cpu.opcode_mapper();
        assert_eq!(cpu.v3, 0xFF);
        assert_eq!(cpu.pc, 0x202);
    }

    // --- add_7xkk ---

    #[test]
    fn test_add_7xkk() {
        let mut cpu = make_cpu();
        cpu.v0 = 10;
        write_opcode(&mut cpu, 0x7005); // V0 += 5
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 15);
    }

    #[test]
    fn test_add_7xkk_wraps() {
        let mut cpu = make_cpu();
        cpu.v0 = 0xFF;
        write_opcode(&mut cpu, 0x7001); // V0 += 1, should wrap
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 0x00);
    }

    // --- load_8xy0 ---

    #[test]
    fn test_load_8xy0() {
        let mut cpu = make_cpu();
        cpu.v2 = 0xAB;
        write_opcode(&mut cpu, 0x8120); // V1 = V2
        cpu.opcode_mapper();
        assert_eq!(cpu.v1, 0xAB);
    }

    // --- or_8xy1 ---

    #[test]
    fn test_or_8xy1() {
        let mut cpu = make_cpu();
        cpu.v0 = 0b1010;
        cpu.v1 = 0b0101;
        write_opcode(&mut cpu, 0x8011); // V0 = V0 | V1
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 0b1111);
    }

    // --- and_8xy2 ---

    #[test]
    fn test_and_8xy2() {
        let mut cpu = make_cpu();
        cpu.v0 = 0b1110;
        cpu.v1 = 0b0111;
        write_opcode(&mut cpu, 0x8012); // V0 = V0 & V1
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 0b0110);
    }

    // --- xor_8xy3 ---

    #[test]
    fn test_xor_8xy3() {
        let mut cpu = make_cpu();
        cpu.v0 = 0b1100;
        cpu.v1 = 0b1010;
        write_opcode(&mut cpu, 0x8013); // V0 = V0 ^ V1
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 0b0110);
    }

    // --- add_8xy4 ---

    #[test]
    fn test_add_8xy4_no_carry() {
        let mut cpu = make_cpu();
        cpu.v0 = 10;
        cpu.v1 = 20;
        write_opcode(&mut cpu, 0x8014); // V0 = V0 + V1
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 30);
        assert_eq!(cpu.vf, 0);
    }

    #[test]
    fn test_add_8xy4_carry() {
        let mut cpu = make_cpu();
        cpu.v0 = 0xFF;
        cpu.v1 = 0x02;
        write_opcode(&mut cpu, 0x8014);
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 0x01);
        assert_eq!(cpu.vf, 1);
    }

    // --- sub_8xy5 (NOTE: borrow based on register index, not value - ported as-is from Gleam) ---

    #[test]
    fn test_sub_8xy5_x_gt_y_index() {
        let mut cpu = make_cpu();
        cpu.v2 = 10;
        cpu.v1 = 3;
        // x=2, y=1 -> index 2 > 1 -> borrow=true, result = Vx - Vy
        write_opcode(&mut cpu, 0x8215);
        cpu.opcode_mapper();
        assert_eq!(cpu.v2, 7);
        assert_eq!(cpu.vf, 1);
    }

    #[test]
    fn test_sub_8xy5_x_lt_y_index() {
        let mut cpu = make_cpu();
        cpu.v1 = 10;
        cpu.v2 = 3;
        // x=1, y=2 -> index 1 < 2 -> borrow=false, result = 0
        write_opcode(&mut cpu, 0x8125);
        cpu.opcode_mapper();
        assert_eq!(cpu.v1, 0);
        assert_eq!(cpu.vf, 0);
    }

    // --- shr_8xy6 ---

    #[test]
    fn test_shr_8xy6_lsb_set() {
        let mut cpu = make_cpu();
        cpu.v0 = 0b00000101;
        write_opcode(&mut cpu, 0x8006);
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 0b00000010);
        assert_eq!(cpu.vf, 1);
    }

    #[test]
    fn test_shr_8xy6_lsb_clear() {
        let mut cpu = make_cpu();
        cpu.v0 = 0b00000100;
        write_opcode(&mut cpu, 0x8006);
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 0b00000010);
        assert_eq!(cpu.vf, 0);
    }

    // --- subn_8xy7 (NOTE: same borrow index bug as sub_8xy5) ---

    #[test]
    fn test_subn_8xy7_x_gt_y_index() {
        let mut cpu = make_cpu();
        cpu.v2 = 3;
        cpu.v1 = 10;
        // x=2, y=1 -> index 2 > 1 -> borrow=true, result = Vy - Vx
        write_opcode(&mut cpu, 0x8217);
        cpu.opcode_mapper();
        assert_eq!(cpu.v2, 7);
        assert_eq!(cpu.vf, 1);
    }

    // --- shl_8xye (NOTE: msb always 0 due to bug ported from Gleam) ---

    #[test]
    fn test_shl_8xye_shifts_left() {
        let mut cpu = make_cpu();
        cpu.v0 = 0b00000010;
        write_opcode(&mut cpu, 0x800E);
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 0b00000100);
        assert_eq!(cpu.vf, 0); // always 0 due to known bug
    }

    // --- skip_next_9xy0 ---

    #[test]
    fn test_skip_next_9xy0_not_equal_skips() {
        let mut cpu = make_cpu();
        cpu.v0 = 1;
        cpu.v1 = 2;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0x9010);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before + 2);
    }

    #[test]
    fn test_skip_next_9xy0_equal_no_skip() {
        let mut cpu = make_cpu();
        cpu.v0 = 5;
        cpu.v1 = 5;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0x9010);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before);
    }

    // --- set_annn ---

    #[test]
    fn test_set_annn() {
        let mut cpu = make_cpu();
        write_opcode(&mut cpu, 0xA123);
        cpu.opcode_mapper();
        assert_eq!(cpu.vi, 0x123);
        assert_eq!(cpu.pc, 0x202);
    }

    // --- set_bnnn ---

    #[test]
    fn test_set_bnnn() {
        let mut cpu = make_cpu();
        cpu.v0 = 0x10;
        write_opcode(&mut cpu, 0xB200);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, 0x210);
    }

    // --- rnd_cxkk ---

    #[test]
    fn test_rnd_cxkk_masked_by_kk() {
        let mut cpu = make_cpu();
        write_opcode(&mut cpu, 0xC000); // Vx = rand & 0x00, always 0
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 0);
    }
}
