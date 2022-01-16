use owo_colors::OwoColorize;
use sdl2::render::Texture;

const PIXEL: &str = "██";

pub trait ACRenderer {
    type RenderResult: Sized;
    type RenderArgs: Sized;

    /// Creates a new instance of the renderer, with all pixels set to false
    fn new() -> Self;

    /// Flips the pixel at x, y and returns if it was erased (colision detection)
    fn flip_pixel(&mut self, x: i8, y: i8) -> bool;

    /// Sets the pixel at x, y to v (mostly for debugging)
    fn set_pixel(&mut self, x: i8, y: i8, v: bool);

    /// Clears the screen, setting all pixels to false
    fn clear(&mut self);

    /// Renders the screen to the desired output format
    fn render(&mut self, args: &mut Self::RenderArgs) -> Self::RenderResult;
}

/// A implementation of ACRenderer that renders to a string for terminal use
pub struct ACStringRenderer {
    pixels: [[bool; 64]; 32],
    //for caching the last render
    last_pixels: Option<[[bool; 64]; 32]>,
    last_render: Option<String>,
}

impl ACRenderer for ACStringRenderer {
    type RenderResult = String;
    type RenderArgs = ();

    fn new() -> Self {
        Self {
            pixels: [[false; 64]; 32],
            last_pixels: None,
            last_render: None,
        }
    }

    fn flip_pixel(&mut self, mut x: i8, mut y: i8) -> bool {
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

    fn set_pixel(&mut self, mut x: i8, mut y: i8, v: bool) {
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

        self.pixels[y as usize][x as usize] = v;
    }

    fn clear(&mut self) {
        self.pixels = [[false; 64]; 32]
    }

    fn render(&mut self, _: &mut ()) -> String {
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

/// A renderer for rendering to a SDL2 texture
pub struct ACSDL2Renderer {
    pixels: [[bool; 64]; 32],
    //for caching the last render
    last_pixels: Option<[[bool; 64]; 32]>,
    last_render: Option<String>,
}

impl ACRenderer for ACSDL2Renderer {
    type RenderResult = ();
    type RenderArgs = Texture<'static>;

    fn new() -> Self {
        Self {
            pixels: [[false; 64]; 32],
            last_pixels: None,
            last_render: None,
        }
    }

    fn flip_pixel(&mut self, mut x: i8, mut y: i8) -> bool {
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

    fn set_pixel(&mut self, mut x: i8, mut y: i8, v: bool) {
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

        self.pixels[y as usize][x as usize] = v;
    }

    fn clear(&mut self) {
        self.pixels = [[false; 64]; 32]
    }

    fn render(&mut self, texture: &mut Texture) {
        texture.with_lock(None, |buffer: &mut [u8], _pitch/* size of a row in bytes */: usize| {
            let mut buf_ptr = 0;
            for row in self.pixels.iter().rev() {
                for px in row {
                    if *px {
                        buffer[buf_ptr] = 255;
                        buffer[buf_ptr+1] = 255;
                        buffer[buf_ptr+2] = 255;
                    } else {
                        buffer[buf_ptr] = 0;
                        buffer[buf_ptr+1] = 0;
                        buffer[buf_ptr+2] = 0;
                    }
                    buf_ptr += 3
                }
            }
        }).expect("Rendered the current frame");
    }
}

pub struct ACEmulator<ACRend>
where
    ACRend: ACRenderer
{
    renderer: ACRend
}

impl<ACRend> ACEmulator<ACRend>
where
    ACRend: ACRenderer
{
    pub fn new(renderer: ACRend) -> Self {
        Self {
            renderer
        }
    }

    pub fn should_bleep(&self) -> bool {
        //TODO finish this
        false
    }

    pub fn update(&mut self, keypad: &crate::keyboard::ACKeyboard) {
        //TODO make this work
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        todo!()
    }

    pub fn render(&mut self, args: &mut ACRend::RenderArgs) -> ACRend::RenderResult {
        self.renderer.render(args)
    }
}