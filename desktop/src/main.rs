use chip8::{
    screen::{SCREEN_HEIGHT, SCREEN_WIDTH},
    CPU,
};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::{env, fs::File, io::Read};

const TICKS_PER_FRAME: usize = 10;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path-to-game");
        std::process::exit(1);
    }

    let sdl_context = sdl2::init().expect("Failed to init SDL2 lib");
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip-8 CPU Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .expect("Failed to create window");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("Failed to build window canvas");
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().expect("Failed to get event pump");

    let mut chip8 = CPU::default();

    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer)
        .expect("Error reading game ROM data");
    chip8.load(&buffer);

    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'gameloop;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, false);
                    }
                }
                _ => (),
            }
        }
        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }
        chip8.tick_timers();
        draw_screen(&chip8, &mut canvas);
    }
}

fn draw_screen(emu: &CPU, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = emu.get_display();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buf.iter().enumerate() {
        // draw white pixels
        if *pixel {
            // convert our 1d array's index into a 2d (x,y) position
            let x = (i % chip8::screen::SCREEN_WIDTH) as u32;
            let y = (i / chip8::screen::SCREEN_WIDTH) as u32;

            // draw a rectangle at (x,y), scaled up by our scale value
            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas
                .fill_rect(rect)
                .expect("Error drawing rectangle for animation");
        }
    }
    canvas.present();
}

fn key2btn(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
