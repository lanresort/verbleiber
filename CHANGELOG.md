# Changelog


## 0.5.0 (unreleased)

- Raised minimum supported Rust version to 1.78.0.

- Updated clap to v4.5.11.

- Updated evdev to v0.12.2.

- Updated log to v0.4.21.

- Updated rodio to v0.17.3.

- Updated simple_logger to v4.3.3.

- Updated toml to v0.8.12.

- Updated ureq to v2.9.6.


## 0.4.0 (2023-10-12)

- Replaced full URL API config property `url` with new property
  `base_url` which leaves out the specific endpoint remainder (which is
  now hard-coded) to support more endpoints without more configuration.

- Changed flow to fetch user data for a tag from the new API endpoint
  instead of keeping local mapping. This obsoletes the configuration
  table `tags_to_user_ids`.

- Obtain filename of sound to play for a user from tag details returned
  by the API. This obsoletes the configuration table `user_sounds`.

- Included party ID in configuration as it became a mandatory parameter
  for the endpoint which sets the status.

- Switched button mappings to match what some sources report as usual
  gamepad button order.

- Replaced whereabouts IDs with human-readable whereabouts names in
  configuration and API protocol.

- Added proper logging.

- Updated clap to v4.4.6.

- Updated flume to v0.11.0.

- Updated ureq to v2.8.0.

- Updated toml to v0.8.2.


## 0.3.0 (2023-05-05)

- Fixed audio playback.


## 0.2.0 (2023-05-05)

- Turned device command-line arguments into configuration properties.

- Grouped API-related configuration properties.

- Show clarifying error message and exit cleanly if input devices could
  not be opened.

- Play sound if user tag is unknown.


## 0.1.0 (2023-05-01)

- First public release
