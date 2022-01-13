use owo_colors::OwoColorize;

const PIXEL: &str = "██";

#[derive(Debug)]
pub struct ACScreen {
    pixels: [[bool; 64]; 32],
    //for caching the last render
    last_pixels: Option<[[bool; 64]; 32]>,
    last_render: Option<String>,
}

impl ACScreen {
    /// Creates a new ACScreen with all pixels set to black
    pub const fn new() -> Self {
        Self {
            pixels: [[false; 64]; 32],
            last_pixels: None,
            last_render: None,
        }
    }

    pub fn flip_pixel(&mut self, mut x: i8, mut y: i8) -> bool {
        if x >= 64 {
            x -= 64;
        } else if x < 0 {
            x += 64;
        }

        if y > 32 {
            y -= 32;
        } else if y < 0 {
            y += 32;
        }

        self.pixels[y as usize][x as usize] = !self.pixels[y as usize][x as usize];

        !self.pixels[y as usize][x as usize]//return if the value at xy was erased
    }

    pub fn clear(&mut self) {
        self.pixels = [[false; 64]; 32]
    }

    pub fn render(&mut self) -> String {
        // caching yay
        if let Some(last_pixels) = &self.last_pixels {
            if *last_pixels == self.pixels {
                return self.last_render.clone().unwrap();
            }
        }

        let mut rendered = String::new();

        let row_end = "\n";
        let px_off = &PIXEL.bg_rgb::<0, 0, 0>().fg_rgb::<0, 0, 0>().to_string();
        let px_on = &PIXEL.fg_rgb::<105, 237, 44>().bg_rgb::<0, 0, 0>().to_string();

        let mut first_row = true;
        for row in self.pixels.iter().rev() {
            if !first_row {
                rendered += row_end;
            } else {
                first_row = false;
            }
            for px in row {
                rendered += if *px {
                    px_on
                } else {
                    px_off
                };
            }
        }

        //caching yay
        self.last_pixels = Some(self.pixels);
        self.last_render = Some(rendered.clone());

        rendered
    }
}