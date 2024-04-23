/*
 * Copyright 2022-2024 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
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

fn open_input_device(device_name: String, label: String) -> Result<Device, String> {
    Device::open(device_name)
        .map_err(|e| format!("Could not open {}: {}", label, e))
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

fn grab_reader_input_device(device: &mut Device, label: String) -> Result<(), String> {
    device
        .grab()
        .map_err(|e| format!("Could not get exclusive access to {}: {}", label, e))
        .map(|()| {
            log::info!("Successfully obtained exclusive access to {}.", label);
        })
}
