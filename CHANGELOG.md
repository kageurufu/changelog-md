# changelog-md

All notable changes to this project will be documented in this file.

The format is derived from [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 1.1.0 - 2025-02-25

First release made through `changelog-md release` 🎉

### Added

- `changelog-md add [added|changed|deprecated|removed|fixed|security] [description]`
- `changelog-md release [--tag=] [--date=] <VERSION> [DESCRIPTION]`

### Changed

- Markdown formatting adjustments, mostly fixing newlines
- Moved revision URLs to a `# Revisions` section

### Fixed

- Changelog::from_path now accepts `impl Into<std::path::PathBuf>`
- Add help string to `changelog-md release`
- `init` default value for `--format` was invalid
- accept `--format yml` as an alias for yaml
- Use pretty toml rendering
- Switch from deprecated serde_yaml to serde_yml
- Add tests for all cli functions
- Fix git urls, tag comparison was reversed

## 1.0.0 - 2025-02-24

Initial release, including the 1.0 Schema specification

# Revisions

- [unreleased] https://github.com/kageurufu/changelog-md/compare/1.1.0...HEAD
- [1.1.0] https://github.com/kageurufu/changelog-md/compare/1.0.0..1.1.0
- [1.0.0] https://github.com/kageurufu/changelog-md/commits/1.0.0
