/*
 * Copyright 2022-2024 Jochen Kupperschmidt
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
    simple_logger::init_with_level(log::Level::Info)?;

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
                log::info!("Tag read: {tag}");

                log::info!("Requesting details for tag {} ...", tag);
                match api_client.get_tag_details(&tag) {
                    Ok(details) => match details {
                        Some(details) => {
                            log::info!(
                                "User for tag {}: {} (ID: {})",
                                details.identifier,
                                details.user.screen_name.unwrap_or("<nameless>".to_string()),
                                details.user.id
                            );
                            let user_id = details.user.id;

                            if let Some(filename) = details.sound_filename {
                                player.play(&filename)?;
                            }

                            log::info!("Awaiting whereabouts for user {user_id} ...");
                            current_user_id = Some(user_id.to_string());
                        }
                        None => {
                            log::info!("Unknown user tag: {tag}");
                            player.play("unknown_user_tag.ogg")?;
                        }
                    },
                    Err(e) => {
                        log::info!("Requesting tag details failed.\n{e}");
                        player.play("oh-nein-netzwerkfehler.ogg")?;
                    }
                };
            }
            UserInput::Button(button_name) => {
                log::info!("Button pressed: {button_name}");

                // Submit if user has identified; ignore if no user has
                // been specified.
                if let Some(user_id) = current_user_id {
                    if let Some(whereabouts_name) = &config.buttons_to_whereabouts.get(&button_name)
                    {
                        log::info!("Submitting whereabouts for user {user_id} ...");

                        let response = api_client.update_status(
                            &user_id,
                            &config.api.party_id,
                            whereabouts_name,
                        );
                        match response {
                            Ok(_) => {
                                log::info!("Status successfully updated.");

                                if let Some(filenames) =
                                    config.whereabouts_sounds.get(*whereabouts_name)
                                {
                                    let filename = choose_random_element(filenames, &mut rng);
                                    player.play(&filename)?;
                                }
                            }
                            Err(e) => {
                                log::info!("Status update failed.\n{e}");
                                player.play("oh-nein-netzwerkfehler.ogg")?;
                            }
                        }
                    }

                    current_user_id = None; // reset
                }
            }
        }
    }

    Ok(())
}

fn choose_random_element(elements: &[String], rng: &mut WyRand) -> String {
    let random_index = rng.generate_range(0..elements.len());
    let element = &elements[random_index];
    element.to_owned()
}
