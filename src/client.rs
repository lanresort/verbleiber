/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use std::path::PathBuf;

use anyhow::Result;
use flume::Receiver;

use crate::api::ApiClient;
use crate::audio::AudioPlayer;
use crate::buttons::Button;
use crate::config::{ApiConfig, PartyConfig};
use crate::events::Event;
use crate::model::{PartyId, UserId, UserMode};
use crate::random::Random;

pub struct Client {
    audio_player: AudioPlayer,
    random: Random,
    api_client: ApiClient,
}

impl Client {
    pub fn new(sounds_path: PathBuf, api_config: &ApiConfig, party_id: &PartyId) -> Result<Self> {
        Ok(Self {
            audio_player: AudioPlayer::new(sounds_path)?,
            random: Random::new(),
            api_client: ApiClient::new(api_config, party_id.clone()),
        })
    }

    pub fn run(
        &self,
        event_receiver: Receiver<Event>,
        party_config: &PartyConfig,
        user_mode: &UserMode,
    ) -> Result<()> {
        self.sign_on()?;

        match user_mode {
            UserMode::SingleUser(user_id) => {
                self.handle_single_user_events(event_receiver, party_config, user_id)?
            }
            UserMode::MultiUser => self.handle_multi_user_events(event_receiver, party_config)?,
        }

        Ok(())
    }

    pub fn handle_single_user_events(
        &self,
        event_receiver: Receiver<Event>,
        party_config: &PartyConfig,
        user_id: &UserId,
    ) -> Result<()> {
        for msg in event_receiver.iter() {
            match msg {
                Event::TagRead { .. } => {
                    log::error!("Unexpected tag read event received.");
                }
                Event::ButtonPressed { button } => {
                    log::debug!("Button pressed: {:?}", button);

                    self.handle_button_press_with_identified_user(
                        user_id.clone(),
                        button,
                        party_config,
                    )?;
                }
                Event::ShutdownRequested => {
                    self.shutdown()?;
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn handle_multi_user_events(
        &self,
        event_receiver: Receiver<Event>,
        party_config: &PartyConfig,
    ) -> Result<()> {
        let mut current_user_id: Option<UserId> = None;

        for msg in event_receiver.iter() {
            match msg {
                Event::TagRead { tag } => {
                    log::debug!("Tag read: {tag}");
                    current_user_id = self.handle_tag_read(&tag)?;
                }
                Event::ButtonPressed { button } => {
                    log::debug!("Button pressed: {:?}", button);

                    // Submit if user has identified; ignore if no user has
                    // been specified.
                    if let Some(user_id) = current_user_id {
                        self.handle_button_press_with_identified_user(
                            user_id,
                            button,
                            party_config,
                        )?;
                        current_user_id = None; // reset
                    }
                }
                Event::ShutdownRequested => {
                    self.shutdown()?;
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn sign_on(&self) -> Result<()> {
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

    pub fn sign_off(&self) -> Result<()> {
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

                    Ok(Some(user_id))
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
        &self,
        user_id: UserId,
        button: Button,
        party_config: &PartyConfig,
    ) -> Result<()> {
        if let Some(whereabouts_name) = &party_config.buttons_to_whereabouts.get(&button) {
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

    fn shutdown(&self) -> Result<()> {
        log::info!("Shutdown requested.");
        self.sign_off()?;
        log::info!("Shutting down ...");
        Ok(())
    }

    fn update_status(&self, user_id: &UserId, whereabouts_name: &str) -> Result<()> {
        self.api_client.update_status(user_id, whereabouts_name)
    }

    fn play_sound(&self, name: &str) -> Result<()> {
        self.audio_player.play(name)
    }

    fn play_random_sound(&self, names: &[String]) -> Result<()> {
        let name = self.random.choose_random_element(names);
        self.play_sound(&name)
    }
}
