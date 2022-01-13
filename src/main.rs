#![allow(dead_code)]

mod screen;
mod keyboard;
mod settings;

use screen::ACScreen;

fn main() {
    let mut scr = ACScreen::new();
    println!("{}\n", scr.render());
    scr.flip_pixel(10, 20);
    println!("{}\n", scr.render());
}
