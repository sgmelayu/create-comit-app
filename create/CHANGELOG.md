# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed
- **BREAKING API CHANGE**: Separate `create` and `start-env` features in 2 binaries and crates.
Meaning that now COMIT Apps must depends on `comit-scripts` package instead of `create-comit-app`.

## [0.7.0] - 2020-01-06

### Changed
- Upgrade all examples and new project to `comit-sdk@0.8.0`.
- To create new project, `new` is not needed in arguments anymore: `yarn create comit-app foo`.
- New binary `comit-scripts` to access other create-comit-app features. E.g.: `comit-scripts start-env`.
- Pressing CTRL+C while `start-env` is running is now handled immediately and not just after the current service is fully started.

## Removed
- Does not automatically clean-up on a panic anymore.

## [0.6.0] - 2019-12-07

### Added
- Windows support :tada:.

## [0.5.2] - 2019-12-06

### Changed
- Upgrade all examples and new project to `comit-sdk@0.7.1`.

## [0.5.1] - 2019-12-02

### Fixed
- Upgrade `separate_apps` example to comit-sdk 0.7.1 to fix maker auto-accept.
- Add code comments in `separate_apps` to describe the behaviour and help app developers understand how to use COMIT SDK.

## [0.5.0] - 2019-11-28

### Changed
- `separate_apps` example now uses the negotiation protocol provided by comit-sdk.
- Upgrade to `comit-rs:0.4.0`
- Upgrade all examples and new project to `comit-sdk@0.7.0`.

## [0.4.0] - 2019-11-20

### Fixed
- Correct log messages in ERC20-BTC example.

### Added
- Updated `btc_eth` example to use the latest comit-js-sdk (0.5.6).
- Added a simple negotiation protocol prototype to the example in `separate_apps`.
- Setting the project name in the `package.json` when running the `new` command.
- Clean up the environment if a panic occurs.
- `force-clean-env` command to stop all services.
- `separate_apps` example where the taker and maker must be started from different terminal to give a better peer-to-peer feeling.

## [0.3.0] - 2019-10-17

### Added
- Deploy an ERC20 contract on the Ethereum node and give tokens to both parties. The address of the token contract is written to the env file as `ERC20_CONTRACT_ADDRESS`.

### Changed
- Migrate parts of the code base to async/await :tada:

## [0.2.1] - 2019-10-04

### Added
- Publish create-comit-app on npmjs.com.

### Changed
- Fix the odd initial Ether amount.

## [0.2.0] - 2019-10-04

### Added
- Ensure that all temporary folders are cleaned up when shutting down start-env.
- Display clear message if start-env is started twice on the same machine.

### Changed
- Move start-env env file to `~/.create-comit-app/env` so that the user does not have to start it from inside the project folder.
- If a signal (e.g. CTRL-C) is sent while `start-env` is booting, it waits for the action in progress (e.g., starting docker container) and then stops and properly cleans up the environment.
- Improve code quality to avoid artifacts remaining after a failure happens while booting start-env.
- Migrate to comit-rs 0.3.0. Only the `cnd` docker is now needed!
- Move the env file from `./.env` to `~/.create-comit-app/env` to improve usability.
- Only pull docker images if they are not present locally, this allows offline usage of the `start-env` command.

## [0.1.0] - 2019-09-26

First Release 🎉

[Unreleased]: https://github.com/comit-network/create-comit-app/compare/0.7.0...HEAD
[0.7.0]: https://github.com/comit-network/create-comit-app/compare/0.6.0...0.7.0
[0.6.0]: https://github.com/comit-network/create-comit-app/compare/0.5.2...0.6.0
[0.5.2]: https://github.com/comit-network/create-comit-app/compare/0.5.1...0.5.2
[0.5.1]: https://github.com/comit-network/create-comit-app/compare/0.5.0...0.5.1
[0.5.0]: https://github.com/comit-network/create-comit-app/compare/0.4.0...0.5.0
[0.4.0]: https://github.com/comit-network/create-comit-app/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/comit-network/create-comit-app/compare/0.2.1...0.3.0
[0.2.1]: https://github.com/comit-network/create-comit-app/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/comit-network/create-comit-app/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/comit-network/create-comit-app/releases/tag/0.1.0