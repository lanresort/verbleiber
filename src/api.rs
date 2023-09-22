/*
 * Copyright 2022-2023 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::{anyhow, Result};
use std::time::Duration;

use crate::config::ApiConfig;

pub(crate) struct ApiClient {
    pub base_url: String,
    pub auth_token: String,
    pub timeout: Duration,
}

impl ApiClient {
    pub(crate) fn new(config: &ApiConfig) -> Self {
        Self {
            base_url: config.base_url.to_owned(),
            auth_token: config.auth_token.to_owned(),
            timeout: Duration::from_secs(config.timeout_in_seconds),
        }
    }

    pub(crate) fn update_status(&self, user_id: &str, whereabouts_id: &str) -> Result<()> {
        let url = format!("{}/set_status", &self.base_url);
        let authz_value = format!("Bearer {}", &self.auth_token);

        ureq::post(&url)
            .timeout(self.timeout)
            .set("Authorization", &authz_value)
            .send_json(ureq::json!({"user_id": &user_id, "whereabouts_id": whereabouts_id}))
            .map_err(|e| anyhow!("Network error: {}", e))
            .map(|_| Ok(()))?
    }
}
