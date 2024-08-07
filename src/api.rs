/*
 * Copyright 2022-2024 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::time::Duration;
use ureq::Error;

use crate::config::ApiConfig;

pub(crate) struct ApiClient {
    pub base_url: String,
    pub auth_token: String,
    pub timeout: Duration,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TagDetails {
    pub identifier: String,
    pub user: TagUser,
    pub sound_filename: Option<String>,
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
            auth_token: config.auth_token.to_owned(),
            timeout: Duration::from_secs(config.timeout_in_seconds),
        }
    }

    pub(crate) fn get_tag_details(&self, tag: &str) -> Result<Option<TagDetails>> {
        let url = format!("{}/tags/{}", &self.base_url, tag);
        let authz_value = format!("Bearer {}", &self.auth_token);
        let request = ureq::get(&url)
            .timeout(self.timeout)
            .set("Authorization", &authz_value);

        match request.call() {
            Ok(response) => response
                .into_json::<TagDetails>()
                .map_err(|e| anyhow!("JSON error: {}", e))
                .map(Some),
            Err(Error::Status(404, _)) => Ok(None),
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
        let authz_value = format!("Bearer {}", self.auth_token);

        match ureq::post(&url)
            .timeout(self.timeout)
            .set("Authorization", &authz_value)
            .send_json(ureq::json!({
                "user_id": user_id,
                "party_id": party_id,
                "whereabouts_name": whereabouts_name,
            })) {
            Ok(_) => Ok(()),
            Err(Error::Status(code, response)) => Err(anyhow!(
                "API error: {} {}: {}",
                code,
                response.status_text().to_string(),
                response.into_string()?
            )),
            Err(e) => Err(anyhow!("Network error: {}", e)),
        }
    }
}
