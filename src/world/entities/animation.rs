use std::{
    collections::HashMap,
    hash::Hash
};

use raylib::prelude::*;

/// Collection of different animations which can be switched between.
pub struct Set<AnimationKey> {
    animations: HashMap<AnimationKey, Animation>,
    current_animation_key: AnimationKey
}

impl<AnimationKey: Hash + Eq> Set<AnimationKey> {
    fn current_animation(&self) -> &Animation {
        self.animations.get(&self.current_animation_key)
    }
}

struct Animation {
    width: u32,
    height: u32,
    frames: Vec<Frame>,
    index: usize
}

struct Frame {
    x: u32,
    y: u32,
    colour: Color
}