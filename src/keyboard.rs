use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash)]
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