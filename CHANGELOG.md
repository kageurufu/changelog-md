# changelog-md

All notable changes to this project will be documented in this file.

The format is derived from [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 1.1.2 - 2025-03-20

Minor release, cleaning up some defaults

### Added

- Attempt to find the current path's git remote url during init
- `yank` command to mark a version as yanked

### Changed

- Don't write null descriptions or yank reasons
- Use mise for build tools, task management, and pre-commit hooks
- Don't write a fake version for a default changelog, instead add an unreleased "added - started using changelog-md"

### Fixed

- Error when attempting to release the same version twice

## 1.1.1 - 2025-02-25

Documentation update

### Added

- documentation for all public members

### Fixed

- Add repository, keywords, and categories to Cargo.toml

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

- [unreleased] <https://github.com/kageurufu/changelog-md/compare/1.1.2...HEAD>
- [1.1.2] <https://github.com/kageurufu/changelog-md/compare/1.1.1..1.1.2>
- [1.1.1] <https://github.com/kageurufu/changelog-md/compare/1.1.0..1.1.1>
- [1.1.0] <https://github.com/kageurufu/changelog-md/compare/1.0.0..1.1.0>
- [1.0.0] <https://github.com/kageurufu/changelog-md/commits/1.0.0>
