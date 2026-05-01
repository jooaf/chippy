# Chippy
This project was used to learn about the Chip8 emulator based on this reference: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM .

The emulator relies on [Minifb](https://github.com/emoon/rust_minifb) for the graphics, [Clap](https://github.com/clap-rs/clap) for the CLI, and [rand](https://github.com/rust-random/rand) for the CXKK opecode.

## About this implementation
- Cpu runs on a separate thread at 500 Hz, or 500 instructions a second (2ms per instruction).
- Window runs at 60fps.
- Runs off binary .rom and .ch8 files.

## Keypad
I have mapped the Original CHIP-8 Keypad:
| 1 | 2 | 3 | C |
|---|---|---|---|
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
| A | 0 | B | F |

to this: 
| 1 | 2 | 3 | 4 |
|---|---|---|---|
| Q | W | E | R |
| A | S | D | F |
| Z | X | C | V |

## CLI
```sh
Usage: chippy <ROM>

Arguments:
  <ROM>  Path to the ROM file

Options:
  -h, --help  Print help
```
