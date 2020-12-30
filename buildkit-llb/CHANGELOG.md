# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2020-03-04
### Changed
- Update `buildkit-proto` dependency to use `tonic` for gRPC.

## [0.1.3] - 2020-01-24
### Added
- `Mount::OptionalSshAgent` to mount the host SSH agent socket with `docker build --ssh=default`.

## [0.1.2] - 2019-11-20
### Added
- `ImageSource::with_tag` method.

### Changed
- `Source::image` behavior to conform Docker.

## [0.1.1] - 2019-10-22
### Added
- `GitSource::with_reference` method.
- HTTP source.

## [0.1.0] - 2019-09-24
Initial release.
