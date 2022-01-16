use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ACKey {
    K0,
    K1,
    K2,
    K3,
    K4,
    K5,
    K6,
    K7,
    K8,
    K9,
    KA,
    KB,
    KC,
    KD,
    KE,
    KF,
}

impl ACKey {
    pub fn from_hex(v: u8) -> Option<Self> {
        Some(match v {
            0x0 => Self::K0,
            0x1 => Self::K1,
            0x2 => Self::K2,
            0x3 => Self::K3,
            0x4 => Self::K4,
            0x5 => Self::K5,
            0x6 => Self::K6,
            0x7 => Self::K7,
            0x8 => Self::K8,
            0x9 => Self::K9,
            0xA => Self::KA,
            0xB => Self::KB,
            0xC => Self::KC,
            0xD => Self::KD,
            0xE => Self::KE,
            0xF => Self::KF,
            _ => return None,
        })
    }

    pub fn to_hex(self) -> u8 {
        match self {
            Self::K0 => 0x0,
            Self::K1 => 0x1,
            Self::K2 => 0x2,
            Self::K3 => 0x3,
            Self::K4 => 0x4,
            Self::K5 => 0x5,
            Self::K6 => 0x6,
            Self::K7 => 0x7,
            Self::K8 => 0x8,
            Self::K9 => 0x9,
            Self::KA => 0xA,
            Self::KB => 0xB,
            Self::KC => 0xC,
            Self::KD => 0xD,
            Self::KE => 0xE,
            Self::KF => 0xF,
        }
    }
}

#[derive(Debug)]
pub struct ACKeyboard {
    keys_pressed: HashSet<ACKey>,
}

impl ACKeyboard {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new()
        }
    }

    pub fn is_pressed(&self, key: &ACKey) -> bool {
        self.keys_pressed.contains(key)
    }

    pub fn press(&mut self, key: ACKey) {
        self.keys_pressed.insert(key);
    }

    pub fn release(&mut self, key: ACKey) {
        self.keys_pressed.remove(&key);
    }
}