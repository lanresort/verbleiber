/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use ctrlc;
use evdev::Device;
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
use crate::audio::AudioPlayer;
use crate::model::UserId;
use crate::userinput::{Button, StringReader};

// TODO: Replace `.unwrap()` with `?` in threads.

fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info)?;

    let cli = cli::parse_cli();

    let config = config::load_config(&cli.config_filename)?;

    let reader_input_device = devices::open_input_device_or_exit(
        config.reader_input_device,
        "reader input device".to_string(),
    )?;

    let button_input_device = devices::open_input_device_or_exit(
        config.button_input_device,
        "button input device".to_string(),
    )?;

    let mut rng = WyRand::new();

    let sounds_path = config.sounds_path.clone();
    let player = AudioPlayer::new(sounds_path)?;

    let (tx1, rx): (Sender<Event>, Receiver<Event>) = flume::unbounded();
    let tx2 = tx1.clone();
    let tx3 = tx1.clone();

    ctrlc::set_handler(move || {
        tx3.send(Event::ShutdownRequested)
            .expect("Could not send shutdown signal")
    })
    .expect("Could not set Ctrl-C handler");

    thread::spawn(|| handle_tag_reads(reader_input_device, tx1));
    thread::spawn(|| handle_button_presses(button_input_device, tx2));

    let mut current_user_id: Option<UserId> = None;

    let api_client = ApiClient::new(&config.api);

    sign_on(&api_client, &player)?;

    for msg in rx.iter() {
        match msg {
            Event::TagRead { tag } => {
                log::info!("Tag read: {tag}");

                log::info!("Requesting details for tag {} ...", tag);
                current_user_id = match api_client.get_tag_details(&tag) {
                    Ok(details) => match details {
                        Some(details) => {
                            log::info!(
                                "User for tag {}: {} (ID: {})",
                                details.identifier,
                                details.user.screen_name.unwrap_or("<nameless>".to_string()),
                                details.user.id
                            );
                            let user_id = details.user.id;

                            if let Some(name) = details.sound_name {
                                player.play(&name)?;
                            }

                            log::info!("Awaiting whereabouts for user {user_id} ...");

                            Some(user_id.to_string())
                        }
                        None => {
                            log::info!("Unknown user tag: {tag}");
                            player.play("unknown_user_tag")?;

                            None
                        }
                    },
                    Err(e) => {
                        log::info!("Requesting tag details failed.\n{e}");
                        player.play("oh-nein-netzwerkfehler")?;

                        None
                    }
                };
            }
            Event::ButtonPressed { button } => {
                log::info!("Button pressed: {:?}", button);

                let button_name = match button {
                    Button::Button1 => "button1".to_string(),
                    Button::Button2 => "button2".to_string(),
                    Button::Button3 => "button3".to_string(),
                    Button::Button4 => "button4".to_string(),
                };

                // Submit if user has identified; ignore if no user has
                // been specified.
                if let Some(user_id) = current_user_id {
                    if let Some(whereabouts_name) =
                        &config.party.buttons_to_whereabouts.get(&button_name)
                    {
                        log::info!(
                            "Submitting whereabouts for user {user_id} -> {whereabouts_name} ..."
                        );

                        let response = api_client.update_status(
                            &user_id,
                            &config.party.party_id,
                            whereabouts_name,
                        );
                        match response {
                            Ok(_) => {
                                log::info!("Status successfully updated.");

                                if let Some(sound_names) =
                                    config.party.whereabouts_sounds.get(*whereabouts_name)
                                {
                                    let sound_name = choose_random_element(sound_names, &mut rng);
                                    player.play(&sound_name)?;
                                }
                            }
                            Err(e) => {
                                log::info!("Status update failed.\n{e}");
                                player.play("oh-nein-netzwerkfehler")?;
                            }
                        }
                    }

                    current_user_id = None; // reset
                }
            }
            Event::ShutdownRequested => {
                log::info!("Shutdown requested.");
                sign_off(&api_client, &player)?;
                log::info!("Shutting down ...");
            }
        }
    }

    Ok(())
}

enum Event {
    TagRead { tag: String },
    ButtonPressed { button: Button },
    ShutdownRequested,
}

fn handle_tag_reads(mut device: Device, sender: Sender<Event>) {
    let mut string_reader = StringReader::new();
    loop {
        for event in device.fetch_events().unwrap() {
            if let Some(value) = string_reader.handle_event(event) {
                let event = Event::TagRead {
                    tag: value.to_string(),
                };
                sender.send(event).unwrap();
            }
        }
    }
}

fn handle_button_presses(mut device: Device, sender: Sender<Event>) {
    loop {
        for event in device.fetch_events().unwrap() {
            if let Some(button) = userinput::handle_button_press(event) {
                let event = Event::ButtonPressed { button };
                sender.send(event).unwrap()
            }
        }
    }
}

fn sign_on(api_client: &ApiClient, player: &AudioPlayer) -> Result<()> {
    log::info!("Signing on ...");
    match api_client.sign_on() {
        Ok(()) => log::info!("Signed on."),
        Err(e) => {
            log::info!("Signing on failed.\n{e}");
            player.play("oh-nein-netzwerkfehler")?;
        }
    }
    Ok(())
}

fn sign_off(api_client: &ApiClient, player: &AudioPlayer) -> Result<()> {
    log::info!("Signing off ...");
    match api_client.sign_off() {
        Ok(()) => log::info!("Signed off."),
        Err(e) => {
            log::info!("Signing off failed.\n{e}");
            player.play("oh-nein-netzwerkfehler")?;
        }
    }
    Ok(())
}

fn choose_random_element(elements: &[String], rng: &mut WyRand) -> String {
    let random_index = rng.generate_range(0..elements.len());
    let element = &elements[random_index];
    element.to_owned()
}
