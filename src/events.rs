/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use crate::buttons::Button;

pub(crate) enum Event {
    TagRead { tag: String },
    ButtonPressed { button: Button },
    ShutdownRequested,
}
