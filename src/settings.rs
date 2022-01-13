use sdl2::audio::AudioSpecDesired;

pub struct ACSettings {
    /// https://en.wikipedia.org/wiki/CHIP-8#Notes
    pub fx1e_affects_vf: bool,
    pub target_fps: u16, //its a u16, beacuse lets face it. you dont have a NASA computer. not using a u8 here, because o p t i m i s i m
    pub audio: AudioSpecDesired,
}