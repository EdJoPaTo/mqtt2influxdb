# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- find values in payloads with units like `12.3 °C` (ignore everything after the whitespace)

### Changed

- reduce heap allocations on InfluxDB line creation.
- reduce heap allocations on MQTT message parsing.

## [0.4.0] - 2023-10-17

### Added

- use `--victoria-metrics` when there is no database, organisation or bucket required.
- `topicSegments` shows the amount of segments in the topic.

### Breaking Changes

- rename `topic-1` to `topicE1` (same with `topic-2` and `topic-3`).
  The old name with the - (minus) is still available but deprecated and will be removed in a future release.

## [0.3.3] - 2023-07-05

### Changed

- Update dependencies

### Fixes

- Ensure influx org and bucket are both provided when using the Influx API v2

## [0.3.2] - 2023-02-08

### Changed

- Update to clap v4
- Update dependencies

## [0.3.1] - 2022-08-18

### Changed

- Switch to `current_thread` async worker (tokio) to reduce dependencies and less thread switches

## [0.3.0] - 2022-03-11

## Added

- Show InfluxDB error message (from request body) on error.
- Build deb/rpm packages.

### Changed

- Increase minimum error wait. This reduces the load on the database as it seems to have some errors currently anyway.
- Higher timeout for InfluxDB writes.

### Fixes

- Only attempt to publish finite floats. NaN for example errors anyway.
- Systemd: restart on-failure.

## [0.2.0] - 2022-02-22

### Added

- Handle termination signals (Ctrl-C, SIGTERM, …) and send buffer before closing

### Changed

- Payload float detection is way more performant with known (`on`, `true`) and other strings.
  This looses support for strange formattings like `onLiNe` but still works with formattings like `online`, `Online` and `ONLINE`.

### Fixes

- Empty MQTT messages are no longer assumed as `0.0`. Clearing retained messages for example are empty messages.

## [0.1.0] - 2022-02-19

Initial release
