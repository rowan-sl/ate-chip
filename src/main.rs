mod emulator;
mod keyboard;
mod settings;


use std::path::PathBuf;
use std::fs;
use std::io::Read;
use std::time::{Duration, Instant};

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};

use log::{debug, info, trace};

use rand::{self, RngCore};

use clap::Parser;

use thiserror::Error;

use emulator::ACEmulator;
use settings::ACSettings;
use keyboard::{ACKeyboard, ACKey};

const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;

const NAME: &str = "Ate-Chip";
const VERSION: &str = clap::crate_version!();
const AUTHOR: &str = clap::crate_authors!();
const ABOUT: &str = clap::crate_description!();

const SETTINGS: ACSettings = ACSettings {
    fx1e_affects_vf: false,
    target_fps: 200,
    audio: AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    }
};

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // minecraft ocean
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

#[derive(Error, Debug)]
pub enum ACEmError {
    #[error("Failed to read file: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("{0}")]
    GenericError(String),
}

impl From<String> for ACEmError {
    fn from(v: String) -> Self {
        Self::GenericError(v)
    }
}

#[derive(Parser, Debug)]
#[clap(name = NAME, author = AUTHOR, version = VERSION, about = ABOUT, long_about = None)]
struct Args {
    #[clap(short, long, default_value_t = 8, help = "Sets the scaling factor")]
    scale: u32,
    #[clap(short, long, help = "path to the rom file")]
    rom: PathBuf,
}


pub fn main() -> Result<(), ACEmError> {
    let args = Args::parse();

    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let mut rom = Vec::new();
    fs::OpenOptions::new()
        .read(true)
        .open(args.rom)?
        .read_to_end(&mut rom)?;

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;

    // for beep thing
    let noise_player = audio_subsystem.open_playback(None, &SETTINGS.audio, |spec| {
        // initialize the audio callback
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25,
        }
    })?;

    let window = video_subsystem
        .window(
            "Ate-Chip",
            SCREEN_WIDTH as u32 * args.scale,
            SCREEN_HEIGHT as u32 * args.scale,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();
    let mut tex_display = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGB24,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        )
        .map_err(|e| e.to_string())?;

    let frame_duration = Duration::new(0, 1_000_000_000u32 / SETTINGS.target_fps as u32);

    let mut emulator = ACEmulator::new();
    emulator.load_rom(rom);

    let mut event_pump = sdl_context.event_pump()?;
    let mut timestamp = Instant::now();
    let mut keyboard = ACKeyboard::new();
    'running: loop {
        canvas.clear();
        trace!("frame");
        let mut key_pressed: Option<ACKey> = None;

        if let Some(event) = event_pump.poll_event() {
            match event {
                Event::Quit {..} => {
                    break 'running
                }
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = keycode {
                        let key = match key {
                            Keycode::Num1 => {
                                ACKey::K1
                            }
                            Keycode::Num2 => {
                                ACKey::K2
                            }
                            Keycode::Num3 => {
                                ACKey::K3
                            }
                            Keycode::Num4 => {
                                ACKey::KC
                            }
                            Keycode::Q => {
                                ACKey::K4
                            }
                            Keycode::W => {
                                ACKey::K5
                            }
                            Keycode::E => {
                                ACKey::K6
                            }
                            Keycode::R => {
                                ACKey::KD
                            }
                            Keycode::A => {
                                ACKey::K7
                            }
                            Keycode::S => {
                                ACKey::K8
                            }
                            Keycode::D => {
                                ACKey::K9
                            }
                            Keycode::F => {
                                ACKey::KE
                            }
                            Keycode::Z => {
                                ACKey::KA
                            }
                            Keycode::X => {
                                ACKey::K0
                            }
                            Keycode::C => {
                                ACKey::KB
                            }
                            Keycode::V => {
                                ACKey::KF
                            }
                            _ => {continue;}
                        };
                        keyboard.press(key.clone());
                        key_pressed = Some(key);
                    }
                },
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = keycode {
                        match key {
                            Keycode::Num1 => {
                                keyboard.release(ACKey::K1)
                            }
                            Keycode::Num2 => {
                                keyboard.release(ACKey::K2)
                            }
                            Keycode::Num3 => {
                                keyboard.release(ACKey::K3)
                            }
                            Keycode::Num4 => {
                                keyboard.release(ACKey::KC)
                            }
                            Keycode::Q => {
                                keyboard.release(ACKey::K4)
                            }
                            Keycode::W => {
                                keyboard.release(ACKey::K5)
                            }
                            Keycode::E => {
                                keyboard.release(ACKey::K6)
                            }
                            Keycode::R => {
                                keyboard.release(ACKey::KD)
                            }
                            Keycode::A => {
                                keyboard.release(ACKey::K7)
                            }
                            Keycode::S => {
                                keyboard.release(ACKey::K8)
                            }
                            Keycode::D => {
                                keyboard.release(ACKey::K9)
                            }
                            Keycode::F => {
                                keyboard.release(ACKey::KE)
                            }
                            Keycode::Z => {
                                keyboard.release(ACKey::KA)
                            }
                            Keycode::X => {
                                keyboard.release(ACKey::K0)
                            }
                            Keycode::C => {
                                keyboard.release(ACKey::KB)
                            }
                            Keycode::V => {
                                keyboard.release(ACKey::KF)
                            }
                            _ => {},
                        }
                    }
                }
                _ => {}
            }
        }

        // testing code
        // if keyboard.is_pressed(&ACKey::K1) {
        //     emulator.set_pixel(10, 10, true);
        // } else {
        //     emulator.set_pixel(10, 10, false);
        // }
        emulator.update(&keyboard, key_pressed);
        emulator.renderer.render_to_tex(&mut tex_display);
        if emulator.should_bleep() {
            noise_player.resume();
        } else {
            noise_player.pause();
        }

        canvas.clear();
        canvas.copy(&tex_display, None, None)?;
        canvas.present();
        let now = Instant::now();
        let sleep_dur = frame_duration
            .checked_sub(now.saturating_duration_since(timestamp))
            .unwrap_or(Duration::new(0, 0));
        ::std::thread::sleep(sleep_dur);
        timestamp = now;
    }

    Ok(())
}