use bevy::prelude::*;

#[derive(Debug, Resource, Default)]
pub struct Currency {
    pub amount: i32,
}
