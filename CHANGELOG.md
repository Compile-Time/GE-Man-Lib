# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

* Implement From<Tag> for String.

### Changed

* Provide access to a GeAsset in ReadProgressWrapper::init.

## [0.1.1] - 2022-06-17

### Changed

* Bring dependencies up-to-date

## [0.1.0] - 2022-03-27

### Added

* `GeDownloader` struct for downloading GE Proton or Wine GE releases. This struct implements the `GeDownload` trait
* so mocking for testing purposes is possible.
* `Tag` struct to represent a GitHub release tag of a GE Proton or Wine GE version. This struct also provides a semantic
  versioning representation to make it easier to compare versions to each other.
* `archive::extract_compressed` to extract GE Proton `.tar.gz` or Wine GE `tar.xz` archives.
* `archive::checksums_match` to compare the generated checksum from an archive to its expected checksum.
* `SteamConfig` struct to read and set a GE Proton version in Steam.
* `LutrisConfig` struct to read and set a Wine GE version in Steam.
* Diverse structs that represent the downloaded resources from GitHub.
