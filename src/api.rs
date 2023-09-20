/*
 * Copyright 2022-2023 Jochen Kupperschmidt
 * License: MIT
 */

use std::time::Duration;

pub(crate) fn update_status(
    base_url: &str,
    auth_token: &str,
    user_id: &str,
    whereabouts_id: &str,
    timeout: Duration,
) -> Result<ureq::Response, ureq::Error> {
    let url = format!("{}/set_status", base_url);
    let authz_value = format!("Bearer {}", auth_token);
    ureq::post(&url)
        .timeout(timeout)
        .set("Authorization", &authz_value)
        .send_json(ureq::json!({"user_id": &user_id, "whereabouts_id": whereabouts_id}))
}
