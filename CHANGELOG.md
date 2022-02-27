# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Increase minimum error wait. This reduces the load on the database as it seems to have some errors currently anyway.

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
