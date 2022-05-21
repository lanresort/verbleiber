/*
 * Copyright 2022 Jochen Kupperschmidt
 * License: MIT
 */

use std::time::Duration;

pub(crate) fn update_status(
    api_url: &str,
    api_token: &str,
    user_id: &str,
    whereabouts_id: &str,
    timeout: Duration,
) -> Result<ureq::Response, ureq::Error> {
    let authz_value = format!("Bearer {}", api_token);
    ureq::post(api_url)
        .timeout(timeout)
        .set("Authorization", &authz_value)
        .send_json(ureq::json!({"user_id": &user_id, "whereabouts_id": whereabouts_id}))
}
