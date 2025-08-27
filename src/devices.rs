/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::{Result, anyhow};
use evdev::Device;
use std::process::exit;

pub(crate) fn open_input_device_or_exit(device_name: String, label: String) -> Result<Device> {
    match open_input_device(device_name, label) {
        Ok(device) => Ok(device),
        Err(e) => {
            log::error!("{}", e);
            exit(1);
        }
    }
}

pub(crate) fn open_input_device(device_name: String, label: String) -> Result<Device> {
    Device::open(device_name)
        .map_err(|e| anyhow!("Could not open {}: {}", label, e))
        .and_then(|mut device| {
            log::info!(
                "Opened {} \"{}\".",
                label,
                device.name().unwrap_or("unnamed device")
            );

            grab_reader_input_device(&mut device, label)?;
            Ok(device)
        })
}

fn grab_reader_input_device(device: &mut Device, label: String) -> Result<()> {
    device
        .grab()
        .map_err(|e| anyhow!("Could not get exclusive access to {}: {}", label, e))
        .map(|()| {
            log::info!("Successfully obtained exclusive access to {}.", label);
        })
}
