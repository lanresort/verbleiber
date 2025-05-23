/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use evdev::{EventSummary, EventType, InputEvent, KeyCode};

pub(crate) struct StringReader {
    chars_read: String,
}

impl StringReader {
    pub fn new() -> Self {
        Self {
            chars_read: String::new(),
        }
    }

    pub fn handle_event(&mut self, event: InputEvent) -> Option<String> {
        if !is_key_released(event) {
            return None;
        }

        if let EventSummary::Key(_, key_code, 0) = event.destructure() {
            match key_code {
                KeyCode::KEY_ENTER => {
                    let input = &self.chars_read.as_str().to_owned();

                    self.chars_read.clear();

                    Some(input.to_owned())
                }
                key_code => match get_char(key_code) {
                    Some(ch) => {
                        self.chars_read.push(ch);
                        None
                    }
                    None => None,
                },
            }
        } else {
            None
        }
    }
}

fn get_char(key_code: KeyCode) -> Option<char> {
    match key_code {
        KeyCode::KEY_1 => Some('1'),
        KeyCode::KEY_2 => Some('2'),
        KeyCode::KEY_3 => Some('3'),
        KeyCode::KEY_4 => Some('4'),
        KeyCode::KEY_5 => Some('5'),
        KeyCode::KEY_6 => Some('6'),
        KeyCode::KEY_7 => Some('7'),
        KeyCode::KEY_8 => Some('8'),
        KeyCode::KEY_9 => Some('9'),
        KeyCode::KEY_0 => Some('0'),
        _ => None,
    }
}

pub(crate) fn handle_button_press(event: InputEvent) -> Option<Button> {
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
