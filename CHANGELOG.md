# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0] - 2024-11-13

### Added

- Most of the app's logic is now factorized into a documented library,
  documentation available [here](https://docs.rs/crate/mksls/latest).

### Changed

- The app made the promise that the output would *always* be a sequence of lines
  of the form:

  ```text
  (<action>) <link> -> <target>
  ```

  The purpose of maintaining such an output was to able you to, if desired,
  redirect the output somewhere for history of what was done.

  However, this promise was not kept, because most uses of the app are
  interactive and reliably cleaning the output is too cumbersome.

  That property of the output was not that useful anyway, so now we don't
  maintain it anymore. Traces of interactive inputs/outputs will remain
  alongside the "feedback" lines in the format above.

- The license is now available in text format instead of Markdown (see LICENSE).

## [1.0.1] - 2024-07-07

### Changed

- Fill uncomplete README regarding installation methods.

## [1.0.0] - 2024-04-17

### Added

- Implementation of a program that creates symlinks specified in text files by the user.
  Only functional on unix systems.

[1.0.0]: https://github.com/yanns1/mksls/releases/tag/v1.0.0
[1.0.1]: https://github.com/yanns1/mksls/compare/v1.0.0...v1.0.1
[2.0.0]: https://github.com/yanns1/mksls/compare/v1.0.1...v2.0.0
[unreleased]: https://github.com/yanns1/mksls/compare/v2.0.0...HEAD
