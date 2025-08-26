/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use flume::{Receiver, Sender};
use std::path::PathBuf;
use std::thread;

mod api;
mod audio;
mod buttons;
mod cli;
mod config;
mod devices;
mod events;
mod model;
mod random;
mod tagreader;

use crate::api::{ApiClient, TagDetails};
use crate::audio::AudioPlayer;
use crate::buttons::Button;
use crate::config::ApiConfig;
use crate::events::Event;
use crate::model::UserId;

fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Debug)?;

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

    let mut random = random::Random::new();

    let sounds_path = config.sounds_path.clone();

    let (tx1, rx): (Sender<Event>, Receiver<Event>) = flume::unbounded();
    let tx2 = tx1.clone();
    let tx3 = tx1.clone();

    ctrlc::set_handler(move || handle_ctrl_c(&tx1)).expect("Could not set Ctrl-C handler");

    thread::spawn(|| tagreader::handle_tag_reads(reader_input_device, tx2));
    thread::spawn(|| buttons::handle_button_presses(button_input_device, tx3));

    let mut current_user_id: Option<UserId> = None;

    let client = Client::new(sounds_path, &config.api, config.party.party_id)?;

    client.sign_on()?;

    for msg in rx.iter() {
        match msg {
            Event::TagRead { tag } => {
                log::debug!("Tag read: {tag}");

                log::debug!("Requesting details for tag {} ...", tag);
                current_user_id = match client.get_tag_details(&tag) {
                    Ok(details) => match details {
                        Some(details) => {
                            log::debug!(
                                "User for tag {}: {} (ID: {})",
                                details.identifier,
                                details.user.screen_name.unwrap_or("<nameless>".to_string()),
                                details.user.id
                            );
                            let user_id = details.user.id;

                            if let Some(name) = details.sound_name {
                                client.play_sound(&name)?;
                            }

                            log::debug!("Awaiting whereabouts for user {user_id} ...");

                            Some(user_id.to_string())
                        }
                        None => {
                            log::info!("Unknown user tag: {tag}");
                            client.play_sound("unknown_user_tag")?;

                            None
                        }
                    },
                    Err(e) => {
                        log::warn!("Requesting tag details failed.\n{e}");
                        client.play_sound("oh-nein-netzwerkfehler")?;

                        None
                    }
                };
            }
            Event::ButtonPressed { button } => {
                log::debug!("Button pressed: {:?}", button);

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
                        log::debug!(
                            "Submitting whereabouts for user {user_id} -> {whereabouts_name} ..."
                        );

                        let response = client.update_status(&user_id, whereabouts_name);
                        match response {
                            Ok(_) => {
                                log::debug!("Status successfully updated.");

                                if let Some(sound_names) =
                                    config.party.whereabouts_sounds.get(*whereabouts_name)
                                {
                                    let sound_name = random.choose_random_element(sound_names);
                                    client.play_sound(&sound_name)?;
                                }
                            }
                            Err(e) => {
                                log::warn!("Status update failed.\n{e}");
                                client.play_sound("oh-nein-netzwerkfehler")?;
                            }
                        }
                    }

                    current_user_id = None; // reset
                }
            }
            Event::ShutdownRequested => {
                log::info!("Shutdown requested.");
                client.sign_off()?;
                log::info!("Shutting down ...");
                break;
            }
        }
    }

    Ok(())
}

fn handle_ctrl_c(sender: &Sender<Event>) {
    sender
        .send(Event::ShutdownRequested)
        .expect("Could not send shutdown signal")
}

struct Client {
    audio_player: AudioPlayer,
    api_client: ApiClient,
}

impl Client {
    fn new(sounds_path: PathBuf, api_config: &ApiConfig, party_id: String) -> Result<Self> {
        Ok(Self {
            audio_player: AudioPlayer::new(sounds_path)?,
            api_client: ApiClient::new(api_config, party_id),
        })
    }

    fn sign_on(&self) -> Result<()> {
        log::info!("Signing on ...");
        match self.api_client.sign_on() {
            Ok(()) => log::info!("Signed on."),
            Err(e) => {
                log::warn!("Signing on failed.\n{e}");
                self.play_sound("oh-nein-netzwerkfehler")?;
            }
        }
        Ok(())
    }

    fn sign_off(&self) -> Result<()> {
        log::info!("Signing off ...");
        match self.api_client.sign_off() {
            Ok(()) => log::info!("Signed off."),
            Err(e) => {
                log::warn!("Signing off failed.\n{e}");
                self.play_sound("oh-nein-netzwerkfehler")?;
            }
        }
        Ok(())
    }

    fn get_tag_details(&self, tag: &str) -> Result<Option<TagDetails>> {
        self.api_client.get_tag_details(tag)
    }

    fn update_status(&self, user_id: &str, whereabouts_name: &str) -> Result<()> {
        self.api_client.update_status(user_id, whereabouts_name)
    }

    fn play_sound(&self, name: &str) -> Result<()> {
        self.audio_player.play(name)
    }
}
