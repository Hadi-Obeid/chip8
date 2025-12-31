use sdl2::audio::AudioSpecDesired;
use sdl2::{event::Event};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::path::Path;
use std::time::Duration;
use structopt::StructOpt;
use text_io::read;

use log::{info, warn, SetLoggerError, LevelFilter};

extern crate sdl2;

mod timestep;
mod logging;
mod chip8;
mod audio;

use logging::SimpleLogger;


#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    debug: bool,

}

use crate::chip8::Chip8;
use crate::timestep::FixedTimestep;

// Init logger
static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map( |()| log::set_max_level(LevelFilter::Trace))
}


static BACKGROUND: Color = Color::RGB(0, 0, 0);
static FOREGROUND: Color = Color::RGB(255, 255, 255);

static WIDTH: u32 = 1280;
static HEIGHT: u32 = 640;

pub fn main() -> Result<(), String> {
    init().expect("Failed to initialize Logger!");

    let opt = Opt::from_args();


    let mut CPU = Chip8::new();

    // Timer for delay
    let mut timer_delay = FixedTimestep::new(60);

    // Timer for instructions
    let mut timer_instructions = FixedTimestep::new(60);

    CPU.load_rom(Path::new("./1-chip8-logo.ch8")).expect("Failed to load ROM");

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        // Turn down amount of samples for
        samples: Some(4),
    };

    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        crate::audio::SquareWave {
            phase: 0.0,
            phase_inc: 440.0 / spec.freq as f32,
            volume: 0.55,
        }
    })?;


    let window = video_subsystem
        .window("rust-sdl2 demo: Video", WIDTH, HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(BACKGROUND);
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    //device.resume();

    if opt.debug {
        println!("Debug mode is activated");
        println!("For commands, enter help!");

        CPU.font_test();
    }
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::A), ..} => CPU.sound_timer = 2,
                _ => {}
            }
        }

        canvas.set_draw_color(BACKGROUND);
        canvas.clear();

        // Update CPU audio and video
        device.resume();
        CPU.update(&mut canvas, &device, 20.);
        CPU.update_timers();

        canvas.present();
        if !opt.debug {
            timer_instructions.update(|| { CPU.cycle(); });
        } else {
            let word: String = read!();
            match word.as_str() {
                "quit" | "q" => {
                    break 'running;
                },

                "step" | "s" => {
                    CPU.cycle();
                },
                _ => {},
            }

        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));


    }

    info!("Quitting!");

    Ok(())
}