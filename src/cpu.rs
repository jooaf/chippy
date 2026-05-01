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
    pub display: [u64; 32],
    pub keyboard: [u8; 16],
    pub curr_key: Option<u8>,
    pub trigger_exc_stop: bool,
    pub dt: u8,
    pub error: u8,
}

const FONT_DATA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

impl CPU {
    pub fn new() -> Self {
        let mut memory = [0u8; 0x1000];
        memory[0x000..0x050].copy_from_slice(&FONT_DATA);

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
            memory,
            display: [0; 32],
            keyboard: [0; 16],
            curr_key: None,
            trigger_exc_stop: false,
            dt: 0,
            error: 0,
        }
    }

    pub fn load_program(&mut self, rom: &[u8]) {
        self.memory[0x200..0x200 + rom.len()].copy_from_slice(rom);
    }

    pub fn update_keyboard(&mut self, index: usize, key_down: bool) {
        self.keyboard[index] = key_down as u8;
        self.curr_key = Some(index as u8);
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

    fn set_register(&mut self, reg: u8, value: u8) {
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
    }
    fn set_register_value(&mut self, reg: u8, value: u8) {
        self.set_register(reg, value);
        self.pc += 2;
    }

    // 00E0 - clear display
    fn clear_00e0(&mut self) {
        self.display = [0; 32];
        self.pc += 2;
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
    fn call_2nnn(&mut self, opcode: u16) {
        let addr = (opcode & 0x0FFF) as usize;
        self.stack[self.sp as usize] = (self.pc + 2) as u16;
        self.sp += 1;
        self.pc = addr;
    }

    // 3XKK - skip next instruction if Vx == kk
    // TODO: on False, pc is not advanced (same instruction re-executes unless main loop handles it)
    fn skip_next_eq_3xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let kk = (opcode & 0x00FF) as u8;
        if self.get_register_value(x) == kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // 4XKK - skip next instruction if Vx != kk
    // TODO: on False, pc is not advanced
    fn skip_next_neq_4xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let kk = (opcode & 0x00FF) as u8;
        if self.get_register_value(x) != kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // 5XY0 - skip next instruction if Vx == Vy
    // TODO: on False, pc is not advanced
    fn skip_next_5xy0(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        if self.get_register_value(nibbles.x) == self.get_register_value(nibbles.y) {
            self.pc += 4;
        } else {
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
    fn sub_8xy5(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let vx = self.get_register_value(nibbles.x);
        let vy = self.get_register_value(nibbles.y);
        let borrow = vx > vy;
        let result = vx.wrapping_sub(vy);
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

    // 8XY7 - set Vx = Vy - Vx, VF = NOT borrow (1 if Vy > Vx, else 0)
    fn subn_8xy7(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let vx = self.get_register_value(nibbles.x);
        let vy = self.get_register_value(nibbles.y);
        let no_borrow = vy > vx;
        let result = vy.wrapping_sub(vx);
        self.set_register_value(nibbles.x, result);
        self.vf = if no_borrow { 1 } else { 0 };
    }

    // 8XYE - set Vx = Vx << 1, VF = MSB before shift
    fn shl_8xye(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let vx = self.get_register_value(nibbles.x);
        // 0b10000000 >> 7
        let msb = (vx & 0x80) >> 7;
        self.set_register_value(nibbles.x, vx << 1);
        self.vf = msb;
    }

    // 9XY0 - skip next instruction if Vx != Vy
    fn skip_next_9xy0(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        if self.get_register_value(nibbles.x) != self.get_register_value(nibbles.y) {
            self.pc += 4;
        } else {
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

    fn build_indices_for_display(&mut self, bound: usize, value: usize, incr: usize) -> Vec<usize> {
        let indices: Vec<usize> = Vec::from_iter(value..value + incr);
        let adj_indices = indices
            .into_iter()
            .map(|val| {
                if val > (bound - 1) {
                    return val % bound;
                }
                val
            })
            .collect();
        adj_indices
    }

    pub fn debug_print_display(&self) -> Vec<Vec<u64>> {
        let mut empty_vec = vec![];

        if self.display == [0; 32] {
            vec![0; 32];
        }
        for row in self.display.iter() {
            let mut row_to_vec: Vec<u64> = vec![];
            for i in 0..64 {
                let val = (*row >> (63 - i)) & 1;
                row_to_vec.push(val);
            }
            empty_vec.push(row_to_vec);
        }

        empty_vec
    }

    pub fn pretty_print_display(&self, grid: &Vec<Vec<u64>>) {
        for row in grid {
            let line: String = row
                .iter()
                .map(|&bit| if bit == 1 { '█' } else { ' ' })
                .collect();
            println!("|{}|", line);
        }
    }

    fn get_sprites_per_row(&mut self, b: u8, indices: &Vec<usize>) -> u64 {
        let sprite_repr = Vec::from_iter(0..8 as u8)
            .into_iter()
            .map(|i| (b >> (7 - i)) & 1)
            .collect::<Vec<u8>>();

        let updated_indices = sprite_repr
            .into_iter()
            .zip(indices)
            .filter(|(bit_val, _idx)| *bit_val == 1)
            .map(|(_bit_val, idx)| *idx as u8)
            .collect::<Vec<u8>>();

        updated_indices
            .into_iter()
            .fold(0x0000000000000000, |mask: u64, idx| {
                mask | (1 << (idx as u64))
            })
    }

    // DXYN - DRW Vx, Vy, nibble
    fn drw_dxyn(&mut self, opcode: u16) {
        self.vf = 0;

        let nibbles = ThreeNibbles::new(opcode);
        let vx = self.get_register_value(nibbles.x);
        let vy = self.get_register_value(nibbles.y);
        let num_bytes = nibbles.lowest;
        let y_indices = self.build_indices_for_display(32, vy as usize, num_bytes as usize);
        let x_indices = self.build_indices_for_display(64, vx as usize, 8);

        // assuming that vi is set correctly
        for (byte_pos, y_index) in y_indices.into_iter().enumerate() {
            let curr_byte = self.memory[self.vi as usize + byte_pos];
            let updated_va = self.get_sprites_per_row(curr_byte, &x_indices);
            let curr_row = self.display[y_index as usize];

            let result = updated_va ^ curr_row;
            let erased = curr_row & !result;
            if erased != 0 {
                self.vf = 1
            }
            self.display[y_index as usize] = updated_va ^ curr_row;
        }

        self.pc += 2;
    }

    // Ex9E - SKP Vx
    fn skp_ex9e(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let keyboard_val = self.get_register_value(nibbles.x);

        // guard against bounds
        if !(keyboard_val as usize <= self.keyboard.len() - 1) {
            return;
        }

        if self.keyboard[keyboard_val as usize] == 1 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // ExA1 - SKNP Vx
    fn sknp_exa1(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let keyboard_val = self.get_register_value(nibbles.x);

        // guard against bounds
        if !(keyboard_val as usize <= self.keyboard.len() - 1) {
            return;
        }

        if self.keyboard[keyboard_val as usize] == 0 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // Fx0A - LD Vx, K
    fn ldvx_fx0a(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);

        if self.curr_key.is_some() {
            self.set_register_value(nibbles.x, self.curr_key.unwrap());
            self.trigger_exc_stop = false;
            self.curr_key = None;
        }
    }
    fn set_register_no_advance(&mut self, reg: u8, value: u8) {
        self.set_register(reg, value);
    }

    // Fx1E - ADD I, Vx
    fn add_i_vx_fx1e(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let val = self.get_register_value(nibbles.x);
        self.vi += val as u16;
        self.pc += 2;
    }

    // Fx07 - LD Vx, DT — set Vx = delay timer value
    fn ldvx_fx07(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        self.set_register_value(nibbles.x, self.dt as u8);
    }

    // Fx15 - LD DT, Vx — set delay timer = Vx
    fn lddt_fx15(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        self.dt = self.get_register_value(nibbles.x);
        self.pc += 2;
    }

    // Fx29 - LD F, Vx — set I = location of font sprite for digit Vx
    // size is 8 x 5 for each sprite. this is the reason we use 5 here for vi
    fn ldf_fx29(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let digit = self.get_register_value(nibbles.x);
        self.vi = (digit as u16) * 5;
        self.pc += 2;
    }

    // Fx33 - LD B, Vx — store BCD of Vx at I, I+1, I+2
    fn ldb_fx33(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        let vx = self.get_register_value(nibbles.x);
        self.memory[self.vi as usize] = vx / 100;
        self.memory[self.vi as usize + 1] = (vx / 10) % 10;
        self.memory[self.vi as usize + 2] = vx % 10;
        self.pc += 2;
    }

    // Fx55 - LD [I], Vx — store V0 through Vx in memory starting at I
    fn ldiv_fx55(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        for i in 0..=nibbles.x {
            self.memory[self.vi as usize + i as usize] = self.get_register_value(i);
        }
        self.pc += 2;
    }

    // Fx65 - LD Vx, [I] — read V0 through Vx from memory starting at I
    fn ldvx_fx65(&mut self, opcode: u16) {
        let nibbles = ThreeNibbles::new(opcode);
        for i in 0..=nibbles.x {
            let val = self.memory[self.vi as usize + i as usize];
            self.set_register_no_advance(i, val);
        }
        self.pc += 2;
    }

    fn opcode_mapper(&mut self) {
        let opcode = self.get_instruction();
        let head = ((opcode & 0xF000) >> 12) as u8;
        let last_byte = (opcode & 0x00FF) as u8;
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
                0xd => self.drw_dxyn(opcode),
                0xe => match last_byte {
                    0x9e => self.skp_ex9e(opcode),
                    0xa1 => self.sknp_exa1(opcode),
                    _ => panic!("Unknown EXY opcode: {:#06x}", opcode),
                },
                0xf => match last_byte {
                    0x0a => {
                        self.trigger_exc_stop = true;
                        self.ldvx_fx0a(opcode)
                    }
                    0x07 => self.ldvx_fx07(opcode),
                    0x15 => self.lddt_fx15(opcode),
                    0x18 => self.pc += 2, // no-op since i didn't want to implement the sound timer
                    0x1e => self.add_i_vx_fx1e(opcode),
                    0x29 => self.ldf_fx29(opcode),
                    0x33 => self.ldb_fx33(opcode),
                    0x55 => self.ldiv_fx55(opcode),
                    0x65 => self.ldvx_fx65(opcode),
                    _ => panic!("Unknown FXY opcode: {:#06x}", opcode),
                },
                _ => panic!("Opcode not implemented: {:#06x}", opcode),
            },
        }
    }

    pub fn cycle(&mut self) {
        self.opcode_mapper();
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
        assert_eq!(cpu.stack[0], (original_pc + 2) as u16);
    }

    // --- skip_next_eq_3xkk ---

    #[test]
    fn test_skip_next_eq_3xkk_equal_skips() {
        let mut cpu = make_cpu();
        cpu.v1 = 0x42;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0x3142); // skip if V1 == 0x42
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before + 4);
    }

    #[test]
    fn test_skip_next_eq_3xkk_not_equal_no_skip() {
        let mut cpu = make_cpu();
        cpu.v1 = 0x10;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0x3142); // skip if V1 == 0x42
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before + 2);
    }

    // --- skip_next_neq_4xkk ---

    #[test]
    fn test_skip_next_neq_4xkk_not_equal_skips() {
        let mut cpu = make_cpu();
        cpu.v2 = 0x10;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0x4242); // skip if V2 != 0x42
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before + 4);
    }

    #[test]
    fn test_skip_next_neq_4xkk_equal_no_skip() {
        let mut cpu = make_cpu();
        cpu.v2 = 0x42;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0x4242); // skip if V2 != 0x42
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before + 2);
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
        assert_eq!(cpu.pc, before + 4);
    }

    #[test]
    fn test_skip_next_5xy0_not_equal_no_skip() {
        let mut cpu = make_cpu();
        cpu.v1 = 5;
        cpu.v2 = 6;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0x5120);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before + 2);
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

    // --- sub_8xy5 ---

    #[test]
    fn test_sub_8xy5_vx_gt_vy() {
        let mut cpu = make_cpu();
        cpu.v2 = 10;
        cpu.v1 = 3;
        // Vx=10 > Vy=3 -> no borrow, VF=1, result = 10 - 3
        write_opcode(&mut cpu, 0x8215);
        cpu.opcode_mapper();
        assert_eq!(cpu.v2, 7);
        assert_eq!(cpu.vf, 1);
    }

    #[test]
    fn test_sub_8xy5_vx_lt_vy() {
        let mut cpu = make_cpu();
        cpu.v1 = 3;
        cpu.v2 = 10;
        // Vx=3 < Vy=10 -> borrow, VF=0, result = 0
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

    // --- subn_8xy7 ---

    #[test]
    fn test_subn_8xy7_vy_gt_vx() {
        let mut cpu = make_cpu();
        cpu.v2 = 3;
        cpu.v1 = 10;
        // Vy=10 > Vx=3 -> no borrow, VF=1, result = 10 - 3
        write_opcode(&mut cpu, 0x8217);
        cpu.opcode_mapper();
        assert_eq!(cpu.v2, 7);
        assert_eq!(cpu.vf, 1);
    }

    // --- shl_8xye ---

    #[test]
    fn test_shl_8xye_shifts_left_msb_clear() {
        let mut cpu = make_cpu();
        cpu.v0 = 0b00000010;
        write_opcode(&mut cpu, 0x800E);
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 0b00000100);
        assert_eq!(cpu.vf, 0);
    }

    #[test]
    fn test_shl_8xye_shifts_left_msb_set() {
        let mut cpu = make_cpu();
        cpu.v0 = 0b10000001;
        write_opcode(&mut cpu, 0x800E);
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 0b00000010);
        assert_eq!(cpu.vf, 1);
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

    // --- drw_dxyn ---

    #[test]
    fn test_drw_dxyn_basic() {
        let mut cpu = make_cpu();
        cpu.v0 = 0;
        cpu.v1 = 0;
        cpu.vi = 0x300;
        cpu.memory[0x300] = 0b11110000; // 4 pixels on from left
        write_opcode(&mut cpu, 0xD011); // draw 1 byte at V0, V1
        cpu.opcode_mapper();
        // MSB-first: bits 7,6,5,4 set → x indices 0,1,2,3 → bits 0,1,2,3 in display
        assert_eq!(cpu.display[0], 0b1111);
        assert_eq!(cpu.vf, 0);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_drw_dxyn_collision() {
        let mut cpu = make_cpu();
        cpu.v0 = 0;
        cpu.v1 = 0;
        cpu.vi = 0x300;
        cpu.memory[0x300] = 0b11110000;
        write_opcode(&mut cpu, 0xD011);
        cpu.opcode_mapper(); // first draw, no collision
        assert_eq!(cpu.vf, 0);
        write_opcode(&mut cpu, 0xD011);
        cpu.opcode_mapper(); // second draw, erases pixels
        assert_eq!(cpu.display[0], 0);
        assert_eq!(cpu.vf, 1);
    }

    #[test]
    fn test_drw_dxyn_vf_reset_on_draw() {
        let mut cpu = make_cpu();
        cpu.vf = 1; // pre-set vf
        cpu.v0 = 0;
        cpu.v1 = 0;
        cpu.vi = 0x300;
        cpu.memory[0x300] = 0b11110000;
        write_opcode(&mut cpu, 0xD011);
        cpu.opcode_mapper();
        assert_eq!(cpu.vf, 0); // reset at start of draw
    }

    #[test]
    fn test_drw_dxyn_multiple_rows() {
        let mut cpu = make_cpu();
        cpu.v0 = 0;
        cpu.v1 = 0;
        cpu.vi = 0x300;
        cpu.memory[0x300] = 0b11000000; // pixels 0,1 on
        cpu.memory[0x301] = 0b00110000; // pixels 2,3 on
        write_opcode(&mut cpu, 0xD012); // 2 bytes
        cpu.opcode_mapper();
        assert_eq!(cpu.display[0], 0b11); // x indices 0,1
        assert_eq!(cpu.display[1], 0b1100); // x indices 2,3
    }

    #[test]
    fn test_drw_dxyn_wrap_x() {
        let mut cpu = make_cpu();
        cpu.v0 = 60; // near right edge
        cpu.v1 = 0;
        cpu.vi = 0x300;
        cpu.memory[0x300] = 0xFF; // all 8 pixels on
        write_opcode(&mut cpu, 0xD011);
        cpu.opcode_mapper();
        // x_indices: 60,61,62,63 wrap to 0,1,2,3
        let expected = (1u64 << 60)
            | (1 << 61)
            | (1 << 62)
            | (1 << 63)
            | (1 << 0)
            | (1 << 1)
            | (1 << 2)
            | (1 << 3);
        assert_eq!(cpu.display[0], expected);
    }

    #[test]
    fn test_drw_dxyn_wrap_y() {
        let mut cpu = make_cpu();
        cpu.v0 = 0;
        cpu.v1 = 31; // bottom row
        cpu.vi = 0x300;
        cpu.memory[0x300] = 0xFF;
        cpu.memory[0x301] = 0xFF;
        write_opcode(&mut cpu, 0xD012); // 2 bytes, second wraps to row 0
        cpu.opcode_mapper();
        let expected_row: u64 = (0..8).fold(0, |m, i| m | (1 << i)); // 0xFF
        assert_eq!(cpu.display[31], expected_row);
        assert_eq!(cpu.display[0], expected_row);
    }

    // --- skp_ex9e ---

    #[test]
    fn test_skp_ex9e_key_pressed_skips() {
        let mut cpu = make_cpu();
        cpu.v0 = 5;
        cpu.keyboard[5] = 1;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0xE09E);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before + 2);
    }

    #[test]
    fn test_skp_ex9e_key_not_pressed_no_advance() {
        let mut cpu = make_cpu();
        cpu.v0 = 5;
        cpu.keyboard[5] = 0;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0xE09E);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before);
    }

    #[test]
    fn test_skp_ex9e_key_zero_works() {
        let mut cpu = make_cpu();
        cpu.v0 = 0;
        cpu.keyboard[0] = 1;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0xE09E);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before + 2);
    }

    // --- sknp_exa1 ---

    #[test]
    fn test_sknp_exa1_key_not_pressed_skips() {
        let mut cpu = make_cpu();
        cpu.v0 = 5;
        cpu.keyboard[5] = 0;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0xE0A1);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before + 2);
    }

    #[test]
    fn test_sknp_exa1_key_pressed_no_advance() {
        let mut cpu = make_cpu();
        cpu.v0 = 5;
        cpu.keyboard[5] = 1;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0xE0A1);
        cpu.opcode_mapper();
        assert_eq!(cpu.pc, before);
    }

    // --- ldvx_fx0a ---

    #[test]
    fn test_ldvx_fx0a_key_available_stores_and_advances() {
        let mut cpu = make_cpu();
        cpu.curr_key = Some(7);
        let before = cpu.pc;
        write_opcode(&mut cpu, 0xF00A);
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 7);
        assert_eq!(cpu.trigger_exc_stop, false);
        assert_eq!(cpu.pc, before + 2);
    }

    #[test]
    fn test_ldvx_fx0a_no_key_halts() {
        let mut cpu = make_cpu();
        cpu.curr_key = None;
        let before = cpu.pc;
        write_opcode(&mut cpu, 0xF00A);
        cpu.opcode_mapper();
        assert_eq!(cpu.trigger_exc_stop, true);
        assert_eq!(cpu.pc, before);
    }

    // --- add_i_vx_fx1e ---
    // NOTE: not yet routed in opcode_mapper (0x1e missing from 0xf block), tested directly

    #[test]
    fn test_add_i_vx_fx1e() {
        let mut cpu = make_cpu();
        cpu.vi = 0x100;
        cpu.v1 = 0x10;
        cpu.add_i_vx_fx1e(0xF11E);
        assert_eq!(cpu.vi, 0x110);
        assert_eq!(cpu.pc, 0x202);
    }

    // --- ldvx_fx07 ---

    #[test]
    fn test_ldvx_fx07_reads_delay_timer() {
        let mut cpu = make_cpu();
        cpu.dt = 42;
        write_opcode(&mut cpu, 0xF007);
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 42);
        assert_eq!(cpu.pc, 0x202);
    }

    // --- lddt_fx15 ---

    #[test]
    fn test_lddt_fx15_sets_delay_timer() {
        let mut cpu = make_cpu();
        cpu.v0 = 30;
        write_opcode(&mut cpu, 0xF015);
        cpu.opcode_mapper();
        assert_eq!(cpu.dt, 30);
        assert_eq!(cpu.pc, 0x202);
    }

    // --- ldf_fx29 ---

    #[test]
    fn test_ldf_fx29_sets_i_to_font_location() {
        let mut cpu = make_cpu();
        cpu.v0 = 0xA; // digit A
        write_opcode(&mut cpu, 0xF029);
        cpu.opcode_mapper();
        assert_eq!(cpu.vi, 0xA * 5); // font for A is at 0x32
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_ldf_fx29_digit_zero() {
        let mut cpu = make_cpu();
        cpu.v0 = 0;
        write_opcode(&mut cpu, 0xF029);
        cpu.opcode_mapper();
        assert_eq!(cpu.vi, 0);
    }

    #[test]
    fn test_ldf_fx29_font_data_readable() {
        let mut cpu = make_cpu();
        cpu.v0 = 0;
        write_opcode(&mut cpu, 0xF029);
        cpu.opcode_mapper();
        // font for '0' should be 0xF0, 0x90, 0x90, 0x90, 0xF0
        assert_eq!(cpu.memory[cpu.vi as usize], 0xF0);
        assert_eq!(cpu.memory[cpu.vi as usize + 1], 0x90);
        assert_eq!(cpu.memory[cpu.vi as usize + 4], 0xF0);
    }

    // --- ldb_fx33 ---

    #[test]
    fn test_ldb_fx33_stores_bcd() {
        let mut cpu = make_cpu();
        cpu.v0 = 234; // hundreds=2, tens=3, ones=4
        cpu.vi = 0x300;
        write_opcode(&mut cpu, 0xF033);
        cpu.opcode_mapper();
        assert_eq!(cpu.memory[0x300], 2);
        assert_eq!(cpu.memory[0x301], 3);
        assert_eq!(cpu.memory[0x302], 4);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_ldb_fx33_single_digit() {
        let mut cpu = make_cpu();
        cpu.v0 = 7;
        cpu.vi = 0x300;
        write_opcode(&mut cpu, 0xF033);
        cpu.opcode_mapper();
        assert_eq!(cpu.memory[0x300], 0);
        assert_eq!(cpu.memory[0x301], 0);
        assert_eq!(cpu.memory[0x302], 7);
    }

    // --- ldiv_fx55 ---

    #[test]
    fn test_ldiv_fx55_stores_registers() {
        let mut cpu = make_cpu();
        cpu.v0 = 0xAA;
        cpu.v1 = 0xBB;
        cpu.v2 = 0xCC;
        cpu.vi = 0x300;
        write_opcode(&mut cpu, 0xF255); // store V0-V2
        cpu.opcode_mapper();
        assert_eq!(cpu.memory[0x300], 0xAA);
        assert_eq!(cpu.memory[0x301], 0xBB);
        assert_eq!(cpu.memory[0x302], 0xCC);
        assert_eq!(cpu.pc, 0x202);
    }

    // --- ldvx_fx65 ---

    #[test]
    fn test_ldvx_fx65_loads_registers() {
        let mut cpu = make_cpu();
        cpu.vi = 0x300;
        cpu.memory[0x300] = 0x11;
        cpu.memory[0x301] = 0x22;
        cpu.memory[0x302] = 0x33;
        write_opcode(&mut cpu, 0xF265); // load V0-V2
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 0x11);
        assert_eq!(cpu.v1, 0x22);
        assert_eq!(cpu.v2, 0x33);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_ldvx_fx65_does_not_affect_registers_beyond_x() {
        let mut cpu = make_cpu();
        cpu.vi = 0x300;
        cpu.memory[0x300] = 0x11;
        cpu.v1 = 0xFF; // should not be changed
        write_opcode(&mut cpu, 0xF065); // load only V0
        cpu.opcode_mapper();
        assert_eq!(cpu.v0, 0x11);
        assert_eq!(cpu.v1, 0xFF);
    }
}
