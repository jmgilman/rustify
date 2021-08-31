# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.2] - 2021-08-30

### Added
- Additional HTTP request methods

## [0.4.1] - 2021-08-26

### Added
- Adds `Endpoint`, `Client`, `MiddleWare`, and `Wrapper` to crate root

### Changed
- Removes feature flags for middleware and wrapping
- Various improvements and fixes to documentation
- Changes `Endpoint::Result` to `Endpoint::Response`

## [0.4.0] - 2021-08-25

### Added
- Support for `async`
- Feature flags for blocking, middleware, and wrapping

### Changed
- Blocking clients have been moved to `rustify::blocking` and are disabled by
  default
- All `Endpoint` execution methods are now `async` by default
- Removes internal implementation of `Request` and `Response` and instead opts
  for using `http::Request` and `http::Response`
- Large amounts of internal refactoring to reduce repeated code and create
  smaller function footprints
- The `MiddleWare` and `Wrapper` implementations are now behind a feature flag
  and disabled by default

### Removed
- The `builder` attribute no longer adds `exec()` methods to the builder variant
  of an `Endpoint`

## [0.3.1] - 2021-08-21

### Added
- Execution methods for returning an Endpoint result wrapped in a generic
  wrapper

### Changed
- Endpoints no longer need to implement `Debug`

## [0.3.0] - 2021-08-21

### Added
- Middleware support for Endpoints for mutating requests and responses during
  the execution process
- Initial infrastructure for supporting more than JSON requests/responses
- Support for getting raw responses back using `Endpoint::exec_raw()`
- Support for sending raw requests using `data` attribute
- Documentation for `rustify_derive`
- Compiltation tests for testing `rustify_derive`

### Changed
- Internal refactoring to improve readability and testing
- Moves helper functions of out `Endpoint` scope
- Substitutes `()` for `EmptyEndpointResponse`
- Removes `strip` option and prefers using middleware
- Renames `Endpoint::execute()` to `Endpoint::exec()`

### Removed
- Support for middleware in `ReqwestClient`

## [0.2.0] - 2021-08-18

### Added
- Query parameters can now be specified using the `query` attribute

### Changed
- Response errors try to parse content to be UTF-8 encoded strings instead of raw bytes
- Successul response codes updated to an inclusive range of 200-208

### Fixed
- The `builder` option can now be used with structs that have generics and lifetimes

## [0.1.0] - 2021-08-15

### Added
- Initial release

[unreleased]: https://github.com/jmgilman/rustify/compare/v0.4.2...HEAD
[0.4.2]: https://github.com/jmgilman/rustify/releases/tag/v0.4.2
[0.4.1]: https://github.com/jmgilman/rustify/releases/tag/v0.4.1
[0.4.0]: https://github.com/jmgilman/rustify/releases/tag/v0.4.0
[0.3.1]: https://github.com/jmgilman/rustify/releases/tag/v0.3.1
[0.3.0]: https://github.com/jmgilman/rustify/releases/tag/v0.3.0
[0.2.0]: https://github.com/jmgilman/rustify/releases/tag/v0.2.0
[0.1.0]: https://github.com/jmgilman/rustify/releases/tag/v0.1.0