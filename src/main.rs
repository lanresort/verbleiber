/*
 * Copyright 2022-2023 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use evdev::{EventType, InputEventKind, Key};
use flume::{Receiver, Sender};
use nanorand::{Rng, WyRand};
use std::thread;
use std::time::Duration;

mod api;
mod audio;
mod cli;
mod config;
mod devices;
mod model;

use crate::model::UserId;

// TODO: Replace `.unwrap()` with `?` in threads.

enum Input {
    User(UserId),
    Button(String),
}

fn get_char(key: Key) -> Option<char> {
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

fn main() -> Result<()> {
    let args = cli::parse_args();

    let config = config::load_config(&args.config_filename)?;

    let mut reader_input_device = devices::open_input_device_or_exit(
        config.reader_input_device,
        "reader input device".to_string(),
    )?;

    let mut button_input_device = devices::open_input_device_or_exit(
        config.button_input_device,
        "button input device".to_string(),
    )?;

    let mut rng = WyRand::new();

    let sounds_path = config.sounds_path.clone();
    let player = audio::Player::new(sounds_path);

    let (tx1, rx): (Sender<Input>, Receiver<Input>) = flume::unbounded();
    let tx2 = tx1.clone();

    // RFID/barcode reader
    thread::spawn(move || {
        let mut read_chars = String::new();
        loop {
            for event in reader_input_device.fetch_events().unwrap() {
                // Only handle pressed key events.
                if event.event_type() != EventType::KEY || event.value() == 1 {
                    continue;
                }

                match event.kind() {
                    InputEventKind::Key(Key::KEY_ENTER) => {
                        let input = read_chars.as_str();

                        let user = Input::User(input.to_string());
                        tx1.send(user).unwrap();

                        read_chars.clear();
                    }
                    InputEventKind::Key(key) => {
                        if let Some(ch) = get_char(key) {
                            read_chars.push(ch)
                        }
                    }
                    _ => (),
                }
            }
        }
    });

    // buttons
    thread::spawn(move || {
        loop {
            for event in button_input_device.fetch_events().unwrap() {
                // Only handle pressed key events.
                if event.event_type() != EventType::KEY || event.value() == 1 {
                    continue;
                }

                match event.kind() {
                    InputEventKind::Key(Key::BTN_TOP) => {
                        tx2.send(Input::Button("button1".to_string())).unwrap();
                    }
                    InputEventKind::Key(Key::BTN_TRIGGER) => {
                        tx2.send(Input::Button("button2".to_string())).unwrap();
                    }
                    InputEventKind::Key(Key::BTN_THUMB2) => {
                        tx2.send(Input::Button("button3".to_string())).unwrap();
                    }
                    InputEventKind::Key(Key::BTN_THUMB) => {
                        tx2.send(Input::Button("button4".to_string())).unwrap();
                    }
                    _ => (),
                }
            }
        }
    });

    let mut current_user_id: Option<UserId> = None;

    for msg in rx.iter() {
        match msg {
            Input::User(tag_id) => {
                println!("User tag ID: {tag_id}");

                match config.tags_to_user_ids.get(&tag_id) {
                    Some(user_id) => {
                        if let Some(filename) = config.user_sounds.get(user_id) {
                            player.play(filename)?;
                        }

                        println!("Awaiting whereabouts for user {user_id} ...");
                        current_user_id = Some(user_id.to_string());
                    }
                    None => {
                        println!("Unknown user tag: {tag_id}");
                        player.play("unknown_user_tag.ogg")?;
                    }
                }
            }
            Input::Button(button_name) => {
                println!("Button pressed: {button_name}");

                // Submit if user has identified; ignore if no user has
                // been specified.
                if let Some(user_id) = current_user_id {
                    if let Some(whereabouts_id) = &config.buttons_to_whereabouts.get(&button_name) {
                        println!("Submitting whereabouts for user {user_id}.");

                        let timeout = Duration::from_secs(config.api.timeout_in_seconds);
                        let response = api::update_status(
                            &config.api.url,
                            &config.api.auth_token,
                            &user_id,
                            whereabouts_id,
                            timeout,
                        );
                        match response {
                            Ok(_) => println!("Request successfully submitted."),
                            Err(e) => {
                                println!("Request failed.\n{e}");
                                player.play("oh-nein-netzwerkfehler.ogg")?;
                            }
                        }

                        if let Some(filenames) = config.whereabouts_sounds.get(*whereabouts_id) {
                            let random_index = rng.generate_range(0..filenames.len());
                            let filename = &filenames[random_index];
                            player.play(filename)?;
                        }
                    }

                    current_user_id = None; // reset
                }
            }
        }
    }

    Ok(())
}
