use chip8_emu::cpu::CPU;
use clap::Parser;
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::{
    fs,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

#[derive(Parser)]
#[command(name = "chip8_emu", about = "CHIP-8 Emulator")]
struct Args {
    /// Path to the ROM file
    rom: String,
}

fn main() {
    let args = Args::parse();
    let rom = fs::read(&args.rom).expect("Failed to read ROM file");
    let mut chip8 = CPU::new();
    chip8.load_program(&rom);

    let chip8 = Arc::new(Mutex::new(chip8));
    let chip8_clone = Arc::clone(&chip8);

    thread::spawn(move || {
        let mut delay_counter = 0;
        let mut cur_cycle = 0;

        loop {
            let mut cpu = chip8_clone.lock().unwrap();

            if cur_cycle % 1000 == 999 {
                let text_display = cpu.debug_print_display();
                dbg!("{:?}", cpu.pretty_print_display(&text_display));
                dbg!();
                dbg!();
                dbg!();
            }
            if cpu.keyboard.iter().any(|&k| k != 0) {
                dbg!(
                    "CYCLE {:?} : Here is the current keyboard: {:?}",
                    cur_cycle,
                    cpu.keyboard
                );
            }

            cur_cycle += 1;

            if cpu.dt > 0 && (delay_counter % 8 == 7) {
                cpu.dt -= 1;
            }

            delay_counter += 1;
            if !cpu.trigger_exc_stop {
                cpu.cycle();
            }
            drop(cpu); // release lock before sleeping
            thread::sleep(Duration::from_millis(2)); // instruction interval is 500 instructions per second. 1000 ms / 500 gives us 2ms
        }
    });

    let mut graphics_buffer = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "CHIP-8 Emulator",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X8,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(60);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut chip8 = chip8.lock().unwrap();

        if !window.get_keys().is_empty() {
            dbg!("The window sees these keys: {:?}", window.get_keys());
        }

        for key in window.get_keys_pressed(KeyRepeat::No).iter() {
            match key {
                Key::Key1 => chip8.update_keyboard(1, true),
                Key::Key2 => chip8.update_keyboard(2, true),
                Key::Key3 => chip8.update_keyboard(3, true),
                Key::Key4 => chip8.update_keyboard(12, true),

                Key::Q => chip8.update_keyboard(4, true),
                Key::W => chip8.update_keyboard(5, true),
                Key::E => chip8.update_keyboard(6, true),
                Key::R => chip8.update_keyboard(13, true),

                Key::A => chip8.update_keyboard(7, true),
                Key::S => chip8.update_keyboard(8, true),
                Key::D => chip8.update_keyboard(9, true),
                Key::F => chip8.update_keyboard(14, true),

                Key::Z => chip8.update_keyboard(10, true),
                Key::X => chip8.update_keyboard(0, true),
                Key::C => chip8.update_keyboard(11, true),
                Key::V => chip8.update_keyboard(15, true),
                _ => (),
            }
        }
        for key in window.get_keys_released().iter() {
            match key {
                Key::Key1 => chip8.update_keyboard(1, false),
                Key::Key2 => chip8.update_keyboard(2, false),
                Key::Key3 => chip8.update_keyboard(3, false),
                Key::Key4 => chip8.update_keyboard(12, false),

                Key::Q => chip8.update_keyboard(4, false),
                Key::W => chip8.update_keyboard(5, false),
                Key::E => chip8.update_keyboard(6, false),
                Key::R => chip8.update_keyboard(13, false),

                Key::A => chip8.update_keyboard(7, false),
                Key::S => chip8.update_keyboard(8, false),
                Key::D => chip8.update_keyboard(9, false),
                Key::F => chip8.update_keyboard(14, false),

                Key::Z => chip8.update_keyboard(10, false),
                Key::X => chip8.update_keyboard(0, false),
                Key::C => chip8.update_keyboard(11, false),
                Key::V => chip8.update_keyboard(15, false),
                _ => (),
            }
        }

        for (r, row) in chip8.display.iter().enumerate() {
            for i in 0..64 as usize {
                let pixel = ((row >> i) & 1) as u8;
                let black_or_white = if pixel == 1 { 0x00FFFFFF } else { 0x00000000 };
                graphics_buffer[r * WIDTH + i] = black_or_white;
            }
        }
        // chip8.keyboard = [0; 16];
        // chip8.curr_key = None;

        drop(chip8);
        window
            .update_with_buffer(&graphics_buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
