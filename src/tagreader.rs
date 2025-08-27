/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use evdev::{Device, EventSummary, EventType, InputEvent, KeyCode};
use flume::Sender;

use crate::events::Event;

pub(crate) fn handle_tag_reads(mut device: Device, sender: Sender<Event>) -> Result<()> {
    let mut tag_reader = TagReader::new();
    loop {
        for event in device.fetch_events()? {
            if let Some(value) = tag_reader.handle_event(event) {
                let event = Event::TagRead {
                    tag: value.to_string(),
                };
                sender.send(event)?;
            }
        }
    }
}

struct TagReader {
    chars_read: String,
}

impl TagReader {
    fn new() -> Self {
        Self {
            chars_read: String::new(),
        }
    }

    fn handle_event(&mut self, event: InputEvent) -> Option<String> {
        if !self.is_key_released(event) {
            return None;
        }

        if let EventSummary::Key(_, key_code, 0) = event.destructure() {
            match key_code {
                KeyCode::KEY_ENTER => {
                    let input = &self.chars_read.as_str().to_owned();

                    self.chars_read.clear();

                    Some(input.to_owned())
                }
                key_code => match self.get_char(key_code) {
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

    fn is_key_released(&self, event: InputEvent) -> bool {
        event.event_type() == EventType::KEY && event.value() == 0
    }

    fn get_char(&self, key_code: KeyCode) -> Option<char> {
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
}
