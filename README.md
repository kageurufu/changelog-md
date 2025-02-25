changelog-md
============

Developer-friendly changelog automation, effectively following https://keepachangelog.com/en/1.1.0/

This project makes no attempts at parsing commit history, in my experience this only leads to messy changelogs.

Instead, write your changelog as a YAML, TOML, or even JSON file and easily render it to markdown

## Installation

`cargo install changelog-md`

changelog-md is also available as a library, exposing the Schema objects

## Usage

```sh
# Create an initial CHANGELOG.toml
$ changelog-md init --format toml

# Render to Markdown. Filename is optional
$ changelog-md render CHANGELOG.toml

# Convert from TOML to YAML format. Filename is optional
$ changelog-md convert --format yaml CHANGELOG.toml

# Validate my CHANGELOG is correct. Filename is optional
$ changelog-md validate CHANGELOG.yml

# Write the current schema to a file
$ changelog-md schema CHANGELOG.schema.json
```

## Format

For a working example, see [CHANGELOG.yml](./CHANGELOG.yml), [CHANGELOG.toml](./CHANGELOG.toml), or [CHANGELOG.json](./CHANGELOG.json).

```yaml
title: The heading for my Changelog
description: Markdown description under the title
repository: https://github.com/author/repository

unreleased:
  added:
    - First addition
    - Second addition
  # changed:
  # fixed:
  # deprecated:
  # removed:
  # security:

versions:
  "1.0.0":
    tag: "tag-1.0.0"
    date: 2025-02-24
    description: |
      Optional description of my version
    added:
      - Everything
```

