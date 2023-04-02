use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Progress {
    pub timer: Timer,
}

pub struct ButtonGain {
    pub gain: i32,
}
