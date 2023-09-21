/*
 * Copyright 2022-2023 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use flume::{Receiver, Sender};
use nanorand::{Rng, WyRand};
use std::thread;

mod api;
mod audio;
mod cli;
mod config;
mod devices;
mod model;
mod userinput;

use crate::api::ApiClient;
use crate::model::UserId;
use crate::userinput::{StringReader, UserInput};

// TODO: Replace `.unwrap()` with `?` in threads.

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

    let (tx1, rx): (Sender<UserInput>, Receiver<UserInput>) = flume::unbounded();
    let tx2 = tx1.clone();

    // RFID/barcode reader
    thread::spawn(move || {
        let mut string_reader = StringReader::new();
        loop {
            for event in reader_input_device.fetch_events().unwrap() {
                if let Some(s) = string_reader.handle_event(event) {
                    let user = UserInput::User(s.to_string());
                    tx1.send(user).unwrap();
                }
            }
        }
    });

    // buttons
    thread::spawn(move || loop {
        for event in button_input_device.fetch_events().unwrap() {
            if let Some(button) = userinput::handle_button_press(event) {
                tx2.send(button).unwrap()
            }
        }
    });

    let mut current_user_id: Option<UserId> = None;

    let api_client = ApiClient::new(&config.api);

    for msg in rx.iter() {
        match msg {
            UserInput::User(tag) => {
                println!("User tag ID: {tag}");

                match config.tags_to_user_ids.get(&tag) {
                    Some(user_id) => {
                        if let Some(filename) = config.user_sounds.get(user_id) {
                            player.play(filename)?;
                        }

                        println!("Awaiting whereabouts for user {user_id} ...");
                        current_user_id = Some(user_id.to_string());
                    }
                    None => {
                        println!("Unknown user tag: {tag}");
                        player.play("unknown_user_tag.ogg")?;
                    }
                }
            }
            UserInput::Button(button_name) => {
                println!("Button pressed: {button_name}");

                // Submit if user has identified; ignore if no user has
                // been specified.
                if let Some(user_id) = current_user_id {
                    if let Some(whereabouts_id) = &config.buttons_to_whereabouts.get(&button_name) {
                        println!("Submitting whereabouts for user {user_id}.");

                        let response = api_client.update_status(&user_id, whereabouts_id);
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
