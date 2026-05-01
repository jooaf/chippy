use chip8_emu::cpu::CPU;

fn run_rom(rom: &[u8], cycles: usize) -> CPU {
    let mut cpu = CPU::new();
    cpu.load_program(rom);
    for _ in 0..cycles {
        if !cpu.trigger_exc_stop {
            cpu.cycle();
        }
    }
    cpu
}

/// Helper to get register value by index
fn reg(cpu: &CPU, r: u8) -> u8 {
    match r {
        0x0 => cpu.v0,
        0x1 => cpu.v1,
        0x2 => cpu.v2,
        0x3 => cpu.v3,
        0x4 => cpu.v4,
        0x5 => cpu.v5,
        0x6 => cpu.v6,
        0x7 => cpu.v7,
        0x8 => cpu.v8,
        0x9 => cpu.v9,
        0xa => cpu.va,
        0xb => cpu.vb,
        0xc => cpu.vc,
        0xd => cpu.vd,
        0xe => cpu.ve,
        0xf => cpu.vf,
        _ => panic!("invalid register"),
    }
}

#[test]
fn test_load_immediate() {
    // 6x kk: LD Vx, kk
    // 6001 -> V0 = 1
    // 6142 -> V1 = 42
    let rom = [0x60, 0x01, 0x61, 0x42];
    let cpu = run_rom(&rom, 2);
    assert_eq!(reg(&cpu, 0), 0x01);
    assert_eq!(reg(&cpu, 1), 0x42);
}

#[test]
fn test_add_immediate() {
    // 6005: V0 = 5
    // 7003: V0 += 3
    let rom = [0x60, 0x05, 0x70, 0x03];
    let cpu = run_rom(&rom, 2);
    assert_eq!(reg(&cpu, 0), 8);
}

#[test]
fn test_add_overflow_no_carry() {
    // ADD immediate (7xkk) does NOT set VF
    // 60FF: V0 = 0xFF
    // 7001: V0 += 1  -> wraps to 0
    let rom = [0x60, 0xFF, 0x70, 0x01];
    let cpu = run_rom(&rom, 2);
    assert_eq!(reg(&cpu, 0), 0x00);
    assert_eq!(reg(&cpu, 0xF), 0); // VF untouched
}

#[test]
fn test_add_register_with_carry() {
    // 60FF: V0 = 0xFF
    // 6101: V1 = 0x01
    // 8014: V0 += V1 (with carry -> VF=1)
    let rom = [0x60, 0xFF, 0x61, 0x01, 0x80, 0x14];
    let cpu = run_rom(&rom, 3);
    assert_eq!(reg(&cpu, 0), 0x00);
    assert_eq!(reg(&cpu, 0xF), 1);
}

#[test]
fn test_sub_register_no_borrow() {
    // 6005: V0 = 5
    // 6103: V1 = 3
    // 8015: V0 -= V1 (5-3=2, no borrow -> VF=1)
    let rom = [0x60, 0x05, 0x61, 0x03, 0x80, 0x15];
    let cpu = run_rom(&rom, 3);
    assert_eq!(reg(&cpu, 0), 2);
    assert_eq!(reg(&cpu, 0xF), 1);
}

#[test]
fn test_sub_register_with_borrow() {
    // 6003: V0 = 3
    // 6105: V1 = 5
    // 8015: V0 -= V1 (borrow -> VF=0)
    let rom = [0x60, 0x03, 0x61, 0x05, 0x80, 0x15];
    let cpu = run_rom(&rom, 3);
    assert_eq!(reg(&cpu, 0xF), 0);
}

#[test]
fn test_jump() {
    // 1206: JP 0x206 (skip one instruction)
    // 6001: V0 = 1 (should be skipped)
    // 6042: V0 = 42
    // Program starts at 0x200, so:
    //   0x200: 12 06
    //   0x202: 60 01
    //   0x204: 60 01
    //   0x206: 60 42
    let rom = [0x12, 0x06, 0x60, 0x01, 0x60, 0x01, 0x60, 0x42];
    let cpu = run_rom(&rom, 2);
    assert_eq!(reg(&cpu, 0), 0x42);
}

#[test]
fn test_skip_equal_byte_true() {
    // 6005: V0 = 5
    // 3005: SE V0, 5 -> skip next
    // 6001: V0 = 1 (skipped)
    // 6002: V0 = 2
    let rom = [0x60, 0x05, 0x30, 0x05, 0x60, 0x01, 0x60, 0x02];
    let cpu = run_rom(&rom, 3);
    assert_eq!(reg(&cpu, 0), 2);
}

#[test]
fn test_skip_not_equal_byte_false() {
    // 6005: V0 = 5
    // 4005: SNE V0, 5 -> condition false, don't skip
    // 6001: V0 = 1
    let rom = [0x60, 0x05, 0x40, 0x05, 0x60, 0x01];
    let cpu = run_rom(&rom, 3);
    assert_eq!(reg(&cpu, 0), 1);
}

#[test]
fn test_or_xor_and() {
    // 600F: V0 = 0x0F
    // 61F0: V1 = 0xF0
    // 8011: V0 = V0 | V1
    let rom_or = [0x60, 0x0F, 0x61, 0xF0, 0x80, 0x11];
    let cpu = run_rom(&rom_or, 3);
    assert_eq!(reg(&cpu, 0), 0xFF);

    // 8013: V0 = V0 & V1
    let rom_and = [0x60, 0xFF, 0x61, 0x0F, 0x80, 0x12];
    let cpu = run_rom(&rom_and, 3);
    assert_eq!(reg(&cpu, 0), 0x0F);

    // 8013: V0 = V0 ^ V1
    let rom_xor = [0x60, 0xFF, 0x61, 0x0F, 0x80, 0x13];
    let cpu = run_rom(&rom_xor, 3);
    assert_eq!(reg(&cpu, 0), 0xF0);
}

#[test]
fn test_set_index() {
    // A123: I = 0x123
    let rom = [0xA1, 0x23];
    let cpu = run_rom(&rom, 1);
    assert_eq!(cpu.vi, 0x123);
}

#[test]
fn test_bcd() {
    // 60C8: V0 = 200 (0xC8)
    // A300: I = 0x300
    // F033: BCD of V0 into mem[I..I+3]
    let rom = [0x60, 0xC8, 0xA3, 0x00, 0xF0, 0x33];
    let cpu = run_rom(&rom, 3);
    assert_eq!(cpu.memory[0x300], 2);
    assert_eq!(cpu.memory[0x301], 0);
    assert_eq!(cpu.memory[0x302], 0);
}

#[test]
fn test_store_and_load_registers() {
    // 6001: V0 = 1
    // 6102: V1 = 2
    // 6203: V2 = 3
    // A300: I = 0x300
    // F255: store V0..V2 to mem[0x300..0x302]
    // 6000: V0 = 0 (clear)
    // 6100: V1 = 0
    // 6200: V2 = 0
    // A300: I = 0x300
    // F265: load V0..V2 from mem[0x300..0x302]
    let rom = [
        0x60, 0x01, // V0 = 1
        0x61, 0x02, // V1 = 2
        0x62, 0x03, // V2 = 3
        0xA3, 0x00, // I = 0x300
        0xF2, 0x55, // store V0..V2
        0x60, 0x00, // V0 = 0
        0x61, 0x00, // V1 = 0
        0x62, 0x00, // V2 = 0
        0xA3, 0x00, // I = 0x300
        0xF2, 0x65, // load V0..V2
    ];
    let cpu = run_rom(&rom, 10);
    assert_eq!(reg(&cpu, 0), 1);
    assert_eq!(reg(&cpu, 1), 2);
    assert_eq!(reg(&cpu, 2), 3);
}

#[test]
fn test_call_and_return() {
    // 0x200: 2206  CALL 0x206
    // 0x202: 6001  V0 = 1  (should run after return)
    // 0x204: 0000  (halt / padding)
    // 0x206: 6042  V0 = 0x42
    // 0x208: 00EE  RET
    let rom = [
        0x22, 0x06, // CALL 0x206
        0x60, 0x01, // V0 = 1
        0x00, 0x00, // padding
        0x60, 0x42, // V0 = 0x42
        0x00, 0xEE, // RET
    ];
    // cycles: CALL, V0=0x42, RET, V0=1
    let cpu = run_rom(&rom, 4);
    assert_eq!(reg(&cpu, 0), 1);
}

#[test]
fn test_shl_sets_vf() {
    // 6080: V0 = 0x80
    // 800E: SHL V0 -> VF = 1, V0 = 0
    let rom = [0x60, 0x80, 0x80, 0x0E];
    let cpu = run_rom(&rom, 2);
    assert_eq!(reg(&cpu, 0xF), 1);
    assert_eq!(reg(&cpu, 0), 0x00);
}

#[test]
fn test_shr_sets_vf() {
    // 6001: V0 = 0x01
    // 8006: SHR V0 -> VF = 1, V0 = 0
    let rom = [0x60, 0x01, 0x80, 0x06];
    let cpu = run_rom(&rom, 2);
    assert_eq!(reg(&cpu, 0xF), 1);
    assert_eq!(reg(&cpu, 0), 0x00);
}
