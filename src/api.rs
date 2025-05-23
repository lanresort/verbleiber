/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use ureq::tls::TlsConfig;
use ureq::{Agent, Error};

use crate::config::ApiConfig;

pub(crate) struct ApiClient {
    pub base_url: String,
    pub client_token: String,
    agent: Agent,
}

#[derive(Debug, Serialize)]
pub(crate) struct StatusUpdate {
    pub user_id: String,
    pub party_id: String,
    pub whereabouts_name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TagDetails {
    pub identifier: String,
    pub user: TagUser,
    pub sound_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TagUser {
    pub id: String,
    pub screen_name: Option<String>,
}

impl ApiClient {
    pub(crate) fn new(config: &ApiConfig) -> Self {
        Self {
            base_url: config.base_url.to_owned(),
            client_token: config.client_token.to_owned(),
            agent: Agent::config_builder()
                .timeout_global(Some(Duration::from_secs(config.timeout_in_seconds)))
                .tls_config(
                    TlsConfig::builder()
                        .disable_verification(!config.tls_verify)
                        .build(),
                )
                .build()
                .into(),
        }
    }

    pub(crate) fn sign_on(&self) -> Result<()> {
        let url = format!("{}/client/sign_on", self.base_url);

        match self
            .agent
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.client_token))
            .send_empty()
        {
            Ok(_) => Ok(()),
            Err(Error::StatusCode(code)) => Err(anyhow!("API error: {}", code)),
            Err(e) => Err(anyhow!("Network error: {}", e)),
        }
    }

    pub(crate) fn sign_off(&self) -> Result<()> {
        let url = format!("{}/client/sign_off", self.base_url);

        match self
            .agent
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.client_token))
            .send_empty()
        {
            Ok(_) => Ok(()),
            Err(Error::StatusCode(code)) => Err(anyhow!("API error: {}", code)),
            Err(e) => Err(anyhow!("Network error: {}", e)),
        }
    }

    pub(crate) fn get_tag_details(&self, tag: &str) -> Result<Option<TagDetails>> {
        let url = format!("{}/tags/{}", &self.base_url, tag);

        match self
            .agent
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.client_token))
            .call()
        {
            Ok(mut response) => response
                .body_mut()
                .read_json::<TagDetails>()
                .map_err(|e| anyhow!("JSON error: {}", e))
                .map(Some),
            Err(Error::StatusCode(404)) => Ok(None),
            Err(e) => Err(anyhow!("Network error: {}", e)),
        }
    }

    pub(crate) fn update_status(
        &self,
        user_id: &str,
        party_id: &str,
        whereabouts_name: &str,
    ) -> Result<()> {
        let url = format!("{}/statuses", self.base_url);

        match self
            .agent
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.client_token))
            .send_json(StatusUpdate {
                user_id: user_id.to_string(),
                party_id: party_id.to_string(),
                whereabouts_name: whereabouts_name.to_string(),
            }) {
            Ok(_) => Ok(()),
            Err(Error::StatusCode(code)) => Err(anyhow!("API error: {}", code)),
            Err(e) => Err(anyhow!("Network error: {}", e)),
        }
    }
}
