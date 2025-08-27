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

use crate::api::ApiClient;
use crate::audio::AudioPlayer;
use crate::buttons::Button;
use crate::config::{ApiConfig, PartyConfig};
use crate::events::Event;
use crate::model::UserId;

fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Debug)?;

    let cli = cli::parse_cli();

    let config = config::load_config(&cli.config_filename)?;

    let sounds_path = config.sounds_path.clone();

    let (tx1, rx): (Sender<Event>, Receiver<Event>) = flume::unbounded();
    let tx2 = tx1.clone();
    let tx3 = tx1.clone();

    ctrlc::set_handler(move || handle_ctrl_c(&tx1)).expect("Could not set Ctrl-C handler");

    thread::spawn(|| tagreader::handle_tag_reads(config.reader_input_device, tx2));
    thread::spawn(|| buttons::handle_button_presses(config.button_input_device, tx3));

    let client = Client::new(sounds_path, &config.api, config.party.party_id.to_string())?;

    client.sign_on()?;

    handle_events(rx, client, &config.party)?;

    Ok(())
}

fn handle_events(
    event_receiver: Receiver<Event>,
    mut client: Client,
    party_config: &PartyConfig,
) -> Result<()> {
    let mut current_user_id: Option<UserId> = None;

    for msg in event_receiver.iter() {
        match msg {
            Event::TagRead { tag } => {
                log::debug!("Tag read: {tag}");
                current_user_id = client.handle_tag_read(&tag)?;
            }
            Event::ButtonPressed { button } => {
                log::debug!("Button pressed: {:?}", button);
                // Submit if user has identified; ignore if no user has
                // been specified.
                if let Some(user_id) = current_user_id {
                    client.handle_button_press_with_identified_user(
                        user_id,
                        button,
                        party_config,
                    )?;
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
    random: random::Random,
    api_client: ApiClient,
}

impl Client {
    fn new(sounds_path: PathBuf, api_config: &ApiConfig, party_id: String) -> Result<Self> {
        Ok(Self {
            audio_player: AudioPlayer::new(sounds_path)?,
            random: random::Random::new(),
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

    fn handle_tag_read(&self, tag: &str) -> Result<Option<UserId>> {
        log::debug!("Requesting details for tag {} ...", tag);
        match self.api_client.get_tag_details(tag) {
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
                        self.play_sound(&name)?;
                    }

                    log::debug!("Awaiting whereabouts for user {user_id} ...");

                    Ok(Some(user_id.to_string()))
                }
                None => {
                    log::info!("Unknown user tag: {tag}");
                    self.play_sound("unknown_user_tag")?;

                    Ok(None)
                }
            },
            Err(e) => {
                log::warn!("Requesting tag details failed.\n{e}");
                self.play_sound("oh-nein-netzwerkfehler")?;

                Ok(None)
            }
        }
    }

    fn handle_button_press_with_identified_user(
        &mut self,
        user_id: UserId,
        button: Button,
        party_config: &PartyConfig,
    ) -> Result<()> {
        let button_name = self.get_button_name(button);
        if let Some(whereabouts_name) = &party_config.buttons_to_whereabouts.get(&button_name) {
            log::debug!("Submitting whereabouts for user {user_id} -> {whereabouts_name} ...");

            let response = self.update_status(&user_id, whereabouts_name);
            match response {
                Ok(_) => {
                    log::debug!("Status successfully updated.");

                    if let Some(sound_names) =
                        &party_config.whereabouts_sounds.get(*whereabouts_name)
                    {
                        self.play_random_sound(sound_names)?;
                    }
                }
                Err(e) => {
                    log::warn!("Status update failed.\n{e}");
                    self.play_sound("oh-nein-netzwerkfehler")?;
                }
            }
        }
        Ok(())
    }

    fn get_button_name(&self, button: Button) -> String {
        match button {
            Button::Button1 => "button1".to_string(),
            Button::Button2 => "button2".to_string(),
            Button::Button3 => "button3".to_string(),
            Button::Button4 => "button4".to_string(),
        }
    }

    fn update_status(&self, user_id: &str, whereabouts_name: &str) -> Result<()> {
        self.api_client.update_status(user_id, whereabouts_name)
    }

    fn play_sound(&self, name: &str) -> Result<()> {
        self.audio_player.play(name)
    }

    fn play_random_sound(&mut self, names: &[String]) -> Result<()> {
        let name = self.random.choose_random_element(names);
        self.play_sound(&name)
    }
}
