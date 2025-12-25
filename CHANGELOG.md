# Changelog

## [Unreleased]

[unreleased]: https://github.com/tcurdt/envq/compare/v0.1.0...HEAD

## [0.1.0] - 2024-11-30

[0.1.0]: https://github.com/tcurdt/envq/releases/tag/v0.1.0

### Added

- Initial release of envq
- List all keys in .env files
- Get/set/delete operations for key values
- Get/set/delete operations for inline comments
- Get/set/delete operations for file headers
- Support for stdin/stdout for encrypted file workflows
- Preserves file formatting (blank lines, comments)
- Setting a key preserves its existing comment
- Deleting a key also removes its comment
- Proper error handling for invalid .env file syntax
- Comprehensive unit test coverage
- Homebrew tap support via `tcurdt/homebrew-tap`
- Nix flake support for `nix profile install` and `nix run`
