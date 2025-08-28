/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::{Context, Result};
use evdev::{Device, EventSummary, EventType, InputEvent, KeyCode};
use flume::Sender;
use serde::Deserialize;
use std::collections::HashMap;
use std::thread;

use crate::devices;
use crate::events::Event;

pub(crate) fn handle_button_presses(
    device_name: String,
    buttons_to_key_code_names: HashMap<Button, String>,
    sender: Sender<Event>,
) -> Result<()> {
    let key_codes_to_buttons = map_key_codes_to_buttons(buttons_to_key_code_names)?;

    let device = open_device(device_name)?;

    let button_handler = ButtonHandler::new(key_codes_to_buttons, sender);
    thread::spawn(move || button_handler.run(device));
    Ok(())
}

fn map_key_codes_to_buttons(
    buttons_to_key_code_names: HashMap<Button, String>,
) -> Result<HashMap<KeyCode, Button>> {
    let mut key_codes_to_buttons: HashMap<KeyCode, Button> = HashMap::new();

    for (button, key_code_name) in buttons_to_key_code_names {
        let key_code = find_key_code_by_name(&key_code_name)
            .with_context(|| format!("Unknown button key code name '{}'", key_code_name))?;

        key_codes_to_buttons.insert(key_code, button);
    }

    Ok(key_codes_to_buttons)
}

fn open_device(device_name: String) -> Result<Device> {
    let device_label = "button input device".to_string();
    devices::open_input_device(device_name, device_label)
}

struct ButtonHandler {
    key_codes_to_buttons: HashMap<KeyCode, Button>,
    sender: Sender<Event>,
}

impl ButtonHandler {
    fn new(key_codes_to_buttons: HashMap<KeyCode, Button>, sender: Sender<Event>) -> Self {
        Self {
            key_codes_to_buttons,
            sender,
        }
    }

    fn run(&self, mut device: Device) -> Result<()> {
        loop {
            for event in device.fetch_events()? {
                if let Some(button) = self.handle_button_press(event) {
                    let event = Event::ButtonPressed { button };
                    self.sender.send(event)?;
                }
            }
        }
    }

    fn handle_button_press(&self, event: InputEvent) -> Option<Button> {
        if !self.is_key_released(event) {
            return None;
        }

        let key_code = match event.destructure() {
            EventSummary::Key(_, key_code, _) => Some(key_code),
            _ => None,
        };

        key_code.and_then(|kc| self.find_button_for_key_code(kc))
    }

    fn is_key_released(&self, event: InputEvent) -> bool {
        event.event_type() == EventType::KEY && event.value() == 0
    }

    fn find_button_for_key_code(&self, key_code: KeyCode) -> Option<Button> {
        self.key_codes_to_buttons.get(&key_code).cloned()
    }
}

fn find_key_code_by_name(name: &str) -> Option<KeyCode> {
    match name {
        "trigger" => Some(KeyCode::BTN_TRIGGER),
        "thumb" => Some(KeyCode::BTN_THUMB),
        "thumb2" => Some(KeyCode::BTN_THUMB2),
        "top" => Some(KeyCode::BTN_TOP),
        _ => None,
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Button {
    Button1,
    Button2,
    Button3,
    Button4,
}
