# Changelog for `plotters-wxdragon`

The format of this changelog is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.1.1]

### Fixed

* Add documentation metadata for docs.rs


## [0.1.0]

### Added

* Create a struct `WxBackend` to bridge `wxdragon::DeviceContext` for use with
  plotters. This struct implements the trait `plotters_backend::DrawingBackend`.
* Create an example `x2` to show basic integration between Plotters and
  wxDragon.
* Create an example `text` to show a way to have the application state control
  the plots.
* Add non-regression tests that write to an in-memory device context and
  compare with a reference png image.

[Unreleased]: https://github.com/olivierlacan/keep-a-changelog/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/olivierlacan/keep-a-changelog/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/olivierlacan/keep-a-changelog/releases/tag/v0.1.0
