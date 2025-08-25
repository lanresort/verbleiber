/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use evdev::{Device, EventSummary, EventType, InputEvent, KeyCode};
use flume::Sender;

use crate::events::Event;

pub(crate) fn handle_button_presses(mut device: Device, sender: Sender<Event>) -> Result<()> {
    loop {
        for event in device.fetch_events()? {
            if let Some(button) = handle_button_press(event) {
                let event = Event::ButtonPressed { button };
                sender.send(event)?;
            }
        }
    }
}

fn handle_button_press(event: InputEvent) -> Option<Button> {
    if !is_key_released(event) {
        return None;
    }

    match event.destructure() {
        EventSummary::Key(_, key_code, _) => find_button_for_key_code(key_code),
        _ => None,
    }
}

fn is_key_released(event: InputEvent) -> bool {
    event.event_type() == EventType::KEY && event.value() == 0
}

fn find_button_for_key_code(key_code: KeyCode) -> Option<Button> {
    match key_code {
        KeyCode::BTN_TRIGGER => Some(Button::Button1),
        KeyCode::BTN_THUMB => Some(Button::Button2),
        KeyCode::BTN_THUMB2 => Some(Button::Button3),
        KeyCode::BTN_TOP => Some(Button::Button4),
        _ => None,
    }
}

#[derive(Debug)]
pub(crate) enum Button {
    Button1,
    Button2,
    Button3,
    Button4,
}
