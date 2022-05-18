/*
 * Copyright 2022 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use evdev::{Device, EventType, InputEventKind, Key};
use flume::{Receiver, Sender};
use nanorand::{Rng, WyRand};
use rodio::{OutputStream, Sink};
use std::process::exit;
use std::thread;
mod audio;
mod cli;
mod config;

// TODO: Replace `.unwrap()` with `?` in threads.

type UserId = String;

#[derive(Debug)]
enum WhereaboutsType {
    Present,
    Away,
    Asleep,
}

enum Input {
    User(UserId),
    Whereabouts(WhereaboutsType),
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

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.sleep_until_end();

    let mut reader_input_device = Device::open(&args.reader_input_device)?;
    println!(
        "Opened reader input device \"{}\".",
        reader_input_device.name().unwrap_or("unnamed device")
    );

    match reader_input_device.grab() {
        Ok(_) => println!("Successfully obtained exclusive access to reader input device."),
        Err(error) => {
            eprintln!(
                "Could not get exclusive access to reader input device: {}",
                error
            );
            exit(1);
        }
    }

    let mut button_input_device = Device::open(&args.button_input_device)?;
    println!(
        "Opened button input device \"{}\".",
        button_input_device.name().unwrap_or("unnamed device")
    );

    match button_input_device.grab() {
        Ok(_) => println!("Successfully obtained exclusive access to button input device."),
        Err(error) => {
            eprintln!(
                "Could not get exclusive access to button input device: {}",
                error
            );
            exit(1);
        }
    }

    let mut rng = WyRand::new();

    let player = audio::Player::new(config.sounds_path.clone(), &sink);

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
                        tx2.send(Input::Whereabouts(WhereaboutsType::Present))
                            .unwrap();
                    }
                    InputEventKind::Key(Key::BTN_TRIGGER) => {
                        tx2.send(Input::Whereabouts(WhereaboutsType::Away)).unwrap();
                    }
                    InputEventKind::Key(Key::BTN_THUMB2) => {
                        tx2.send(Input::Whereabouts(WhereaboutsType::Asleep))
                            .unwrap();
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

                if let Some(user_id) = config.tags_to_user_ids.get(&tag_id) {
                    if let Some(filename) = config.user_sounds.get(user_id) {
                        player.play(filename)?;
                    }

                    println!("Awaiting whereabouts for user {user_id} ...");
                    current_user_id = Some(user_id.to_string());
                }
            }
            Input::Whereabouts(t) => {
                println!("Whereabouts: {t:?}");

                match current_user_id {
                    Some(user_id) => {
                        println!("Submitting whereabouts ({t:?}) for user {user_id}.");

                        let whereabouts_key = match t {
                            WhereaboutsType::Present => "present",
                            WhereaboutsType::Away => "away",
                            WhereaboutsType::Asleep => "asleep",
                        };

                        let whereabouts_id = match t {
                            WhereaboutsType::Present => "2ace4e57-e083-490b-ad10-3d214666db57",
                            WhereaboutsType::Away => "568d4a1e-5af0-43cd-8249-832ffc387e4d",
                            WhereaboutsType::Asleep => "92f92f15-fbc1-4e77-bd75-65186f9e4a4d",
                        };

                        let authz_value = format!("Bearer {}", config.api_token);
                        let response = ureq::post(&config.api_url)
                            .set("Authorization", &authz_value)
                            .send_json(ureq::json!({"user_id": &user_id, "whereabouts_id": whereabouts_id}));
                        match response {
                            Ok(_) => println!("Request successfully submitted."),
                            Err(e) => println!("Request failed.\n{e}"),
                        }

                        if let Some(filenames) = config.whereabouts_sounds.get(whereabouts_key) {
                            let random_index = rng.generate_range(0..filenames.len());
                            let filename = &filenames[random_index];
                            player.play(filename)?;
                        }

                        current_user_id = None; // reset
                    }
                    None => (), // Ignore if no user has been specified.
                }
            }
        }
    }

    Ok(())
}