/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use nanorand::{Rng, WyRand};

pub(crate) struct Random {
    rng: WyRand,
}

impl Random {
    pub(crate) fn new() -> Self {
        Self { rng: WyRand::new() }
    }

    pub(crate) fn choose_random_element(&mut self, elements: &[String]) -> String {
        let random_index = self.rng.generate_range(0..elements.len());
        let element = &elements[random_index];
        element.to_owned()
    }
}
