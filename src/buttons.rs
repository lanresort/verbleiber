/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use evdev::{EventSummary, EventType, InputEvent, KeyCode};
use flume::Sender;

use crate::devices;
use crate::events::Event;

pub(crate) fn handle_button_presses(device_name: String, sender: Sender<Event>) -> Result<()> {
    let button_handler = ButtonHandler::new(sender);
    button_handler.run(device_name)?;
    Ok(())
}

struct ButtonHandler {
    sender: Sender<Event>,
}

impl ButtonHandler {
    fn new(sender: Sender<Event>) -> Self {
        Self { sender }
    }

    fn run(&self, device_name: String) -> Result<()> {
        let device_label = "button input device".to_string();
        let mut device = devices::open_input_device_or_exit(device_name, device_label)?;

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

        match event.destructure() {
            EventSummary::Key(_, key_code, _) => self.find_button_for_key_code(key_code),
            _ => None,
        }
    }

    fn is_key_released(&self, event: InputEvent) -> bool {
        event.event_type() == EventType::KEY && event.value() == 0
    }

    fn find_button_for_key_code(&self, key_code: KeyCode) -> Option<Button> {
        match key_code {
            KeyCode::BTN_TRIGGER => Some(Button::Button1),
            KeyCode::BTN_THUMB => Some(Button::Button2),
            KeyCode::BTN_THUMB2 => Some(Button::Button3),
            KeyCode::BTN_TOP => Some(Button::Button4),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) enum Button {
    Button1,
    Button2,
    Button3,
    Button4,
}
