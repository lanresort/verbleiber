/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::{Result, bail};
use flume::{Receiver, Sender};
use simple_logger::SimpleLogger;
use std::path::PathBuf;

mod api;
mod audio;
mod buttons;
mod cli;
mod client;
mod config;
mod devices;
mod events;
mod http;
mod model;
mod random;
mod registration;
mod tagreader;

use crate::client::Client;
use crate::config::PartyConfig;
use crate::events::Event;
use crate::model::{UserId, UserMode};

fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .with_module_level("verbleiber", log::LevelFilter::Debug)
        .init()?;

    let cli = cli::parse_cli();

    match cli.command {
        cli::Command::Register {
            base_url,
            button_count,
            audio_output,
            disable_tls_verification,
        } => registration::register(
            &base_url,
            button_count,
            audio_output,
            disable_tls_verification,
        )?,
        cli::Command::Run { config_filename } => run(config_filename)?,
    }

    Ok(())
}

fn run(config_filename: PathBuf) -> Result<()> {
    let config = config::load_config(&config_filename)?;

    let user_mode = config.get_user_mode();
    match &user_mode {
        UserMode::SingleUser(id) => log::info!("Running in single-user mode for user ID '{id}'."),
        UserMode::MultiUser => log::info!("Running in multi-user mode."),
    }

    let sounds_path = config.sounds_path.clone();

    let (tx1, rx): (Sender<Event>, Receiver<Event>) = flume::unbounded();
    let tx2 = tx1.clone();
    let tx3 = tx1.clone();

    ctrlc::set_handler(move || handle_ctrl_c(&tx1)).expect("Could not set Ctrl-C handler");

    if let UserMode::MultiUser = user_mode {
        match config.reader_input_device {
            Some(device) => tagreader::handle_tag_reads(device, tx2)?,
            None => bail!("No reader device configured, but one is required in multi-user mode."),
        }
    }

    buttons::handle_button_presses(
        config.button_input_device,
        config.buttons_to_key_code_names,
        tx3,
    )?;

    let client = Client::new(sounds_path, &config.api, &config.party.party_id)?;

    client.sign_on()?;

    handle_events(rx, client, &config.party, &user_mode)?;

    Ok(())
}

fn handle_events(
    event_receiver: Receiver<Event>,
    mut client: Client,
    party_config: &PartyConfig,
    user_mode: &UserMode,
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

                match user_mode {
                    UserMode::SingleUser(user_id) => {
                        client.handle_button_press_with_identified_user(
                            user_id.clone(),
                            button,
                            party_config,
                        )?;
                    }
                    UserMode::MultiUser => {
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
                }
            }
            Event::ShutdownRequested => {
                shutdown(&client)?;
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

fn shutdown(client: &Client) -> Result<()> {
    log::info!("Shutdown requested.");
    client.sign_off()?;
    log::info!("Shutting down ...");
    Ok(())
}
