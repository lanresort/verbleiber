/*
 * Copyright 2022-2023 Jochen Kupperschmidt
 * License: MIT
 */

use evdev::{EventType, InputEvent, InputEventKind, Key};

use crate::model::UserId;

pub(crate) enum UserInput {
    User(UserId),
    Button(String),
}

pub(crate) fn get_char(key: Key) -> Option<char> {
    match key {
        Key::KEY_1 => Some('1'),
        Key::KEY_2 => Some('2'),
        Key::KEY_3 => Some('3'),
        Key::KEY_4 => Some('4'),
        Key::KEY_5 => Some('5'),
        Key::KEY_6 => Some('6'),
        Key::KEY_7 => Some('7'),
        Key::KEY_8 => Some('8'),
        Key::KEY_9 => Some('9'),
        Key::KEY_0 => Some('0'),
        _ => None,
    }
}

pub(crate) fn handle_button_press(event: InputEvent) -> Option<UserInput> {
    if !is_key_released(event) {
        return None;
    }

    match event.kind() {
        InputEventKind::Key(key) => find_button_for_key(key),
        _ => None,
    }
}

pub(crate) fn is_key_released(event: InputEvent) -> bool {
    event.event_type() == EventType::KEY && event.value() == 0
}

fn find_button_for_key(key: Key) -> Option<UserInput> {
    match key {
        Key::BTN_TOP => Some("button1".to_string()),
        Key::BTN_TRIGGER => Some("button2".to_string()),
        Key::BTN_THUMB2 => Some("button3".to_string()),
        Key::BTN_THUMB => Some("button4".to_string()),
        _ => None,
    }
    .map(UserInput::Button)
}
