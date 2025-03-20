use anyhow::{anyhow, bail};
use changelog_md::{Changelog, Version};

use clap::{Parser, Subcommand, ValueEnum};
use schemars::schema_for;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    /// Manually specify the path to your changelog
    #[clap(short, long)]
    changelog: Option<std::path::PathBuf>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate an initial CHANGELOG.yml
    Init {
        #[clap(short, long, default_value = "yaml")]
        format: Format,
    },

    /// Convert a CHANGELOG source to another format
    Convert {
        #[clap(long, default_value = "false")]
        force: bool,
        #[clap(short, long, default_value = "yaml")]
        format: Format,
    },

    /// Validate a CHANGELOG
    Validate,

    /// Get the CHANGELOG schema
    Schema {
        /// Destination to write the changelog
        destination: Option<std::path::PathBuf>,
    },

    /// Add an unreleased change
    Add {
        change_type: ChangeType,
        description: String,
    },

    /// Create a new release from all unreleased changes
    Release {
        /// Git Tag, if differs from the version
        #[clap(long)]
        tag: Option<String>,
        /// Release date, defaults to the current date
        #[clap(long)]
        date: Option<String>,

        /// New version name
        version: String,
        /// Release description
        description: Option<String>,
    },

    /// Yank a release
    Yank {
        /// Version to yank
        version: String,
        /// Reason for yanking this version
        reason: String,
    },

    /// Render a CHANGELOG to Markdown
    Render {
        /// Destination path
        destination: Option<std::path::PathBuf>,
    },

    /// Generate release notes for a single version
    ReleaseNotes { version: Option<String> },
}

#[derive(Debug, Clone, ValueEnum)]
enum ChangeType {
    Added,
    Changed,
    Deprecated,
    Removed,
    Fixed,
    Security,
}

#[derive(Debug, Default, Clone, ValueEnum)]
enum Format {
    #[default]
    #[value(alias("yml"))]
    Yaml,
    Toml,
    Json,
}

impl Format {
    pub fn extension(&self) -> &str {
        match self {
            Format::Yaml => "yml",
            Format::Toml => "toml",
            Format::Json => "json",
        }
    }

    pub fn to_string(&self, seed: &Changelog) -> anyhow::Result<String> {
        match self {
            Format::Yaml => seed.to_yaml(),
            Format::Toml => seed.to_toml(),
            Format::Json => seed.to_json(),
        }
    }
}

impl TryFrom<&std::path::PathBuf> for Format {
    type Error = anyhow::Error;

    fn try_from(value: &std::path::PathBuf) -> Result<Self, Self::Error> {
        match value.extension().map(|v| v.to_ascii_lowercase()) {
            Some(ext) if ext == "yml" || ext == "yaml" => Ok(Format::Yaml),
            Some(ext) if ext == "json" => Ok(Format::Json),
            Some(ext) if ext == "toml" => Ok(Format::Toml),
            Some(ext) => Err(anyhow!("Unknown file extension {}", ext.to_string_lossy())),
            _ => Err(anyhow!(
                "Missing file extension for {}",
                value.to_string_lossy()
            )),
        }
    }
}

fn autodetect_source() -> anyhow::Result<std::path::PathBuf> {
    let entries = std::fs::read_dir(".")
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_stem()
                .is_some_and(|s| s.eq_ignore_ascii_case("changelog"))
        })
        .filter(|p| {
            p.extension()
                .map(|s| s.to_ascii_lowercase())
                .is_some_and(|ext| ext == "yml" || ext == "yaml" || ext == "toml" || ext == "json")
        })
        .collect::<Vec<_>>();

    if entries.len() > 1 {
        Err(anyhow!("Multiple changelog source files found"))
    } else {
        entries
            .first()
            .cloned()
            .ok_or(anyhow!("Unable to find changelog source file"))
    }
}

/// Search upwards for a .git/config with `[remote "origin"] url = ...`
fn get_git_remote() -> Option<String> {
    if let Ok(path) = std::env::current_dir() {
        for path in path.ancestors() {
            let git_config = path.join(".git/config");
            if git_config.exists() {
                if let Ok(contents) = std::fs::read_to_string(git_config) {
                    if let Ok(conf) = ini::Ini::load_from_str(&contents) {
                        if let Some(section) = conf.section(Some(r#"remote "origin""#)) {
                            if let Some(url) = section.get("url") {
                                return Some(url.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let changelog_file = {
        match args.changelog {
            Some(filename) => Ok(filename),
            None => autodetect_source(),
        }
    };

    match args.command {
        Command::Init { format } => {
            let filename = changelog_file
                .unwrap_or_else(|_| format!("CHANGELOG.{}", format.extension()).into());

            if filename.exists() {
                Err(anyhow!("{} already exists", filename.display()))
            } else {
                let mut seed = Changelog::default();
                if let Some(url) = get_git_remote() {
                    seed.repository = url;
                };
                let seed = format.to_string(&seed)?;
                eprintln!("Writing initial {}", filename.display());
                std::fs::write(filename, seed)?;

                Ok(())
            }
        }

        Command::Convert { format, force } => {
            let changelog_file = changelog_file?;
            let destination = changelog_file.with_extension(format.extension());

            if destination.exists() && !force {
                return Err(anyhow!("{} already exists", destination.display()));
            }

            let changelog = Changelog::from_path(&changelog_file)?;
            let changelog = format.to_string(&changelog)?;
            eprintln!(
                "Converting {} to {}",
                changelog_file.display(),
                destination.display()
            );
            std::fs::write(destination, changelog)?;

            Ok(())
        }

        Command::Render { destination } => {
            let changelog_file = changelog_file?;
            let changelog = Changelog::from_path(&changelog_file)?;

            let destination = destination.unwrap_or_else(|| changelog_file.with_extension("md"));

            eprintln!(
                "Rendering {} to {}",
                changelog_file.display(),
                destination.display()
            );
            Ok(std::fs::write(destination, format!("{}", &changelog))?)
        }

        Command::ReleaseNotes { version } => {
            let changelog_file = changelog_file?;
            let changelog = Changelog::from_path(&changelog_file)?;

            match version {
                None => {
                    if changelog.unreleased.is_empty() {
                        eprintln!("Warning: No unreleased changes, release notes will be empty");
                    }
                    print!("{}", changelog.unreleased);
                    Ok(())
                }

                Some(version) => {
                    for released_version in &changelog.versions {
                        if released_version.version == version {
                            print!("{}", released_version);
                            return Ok(());
                        }
                    }

                    eprintln!("Currently released versions:");
                    for released_version in &changelog.versions {
                        eprintln!("  {}", released_version.version);
                    }
                    bail!("Could not find version {}", version);
                }
            }
        }

        Command::Validate => {
            Changelog::from_path(&changelog_file?)?;
            println!("No issues found");
            Ok(())
        }

        Command::Schema { destination } => {
            let schema = {
                let mut schema = schema_for!(Changelog);
                let metadata = schema.schema.metadata.as_mut().unwrap();
                metadata.id = Some("https://changelog-md.github.io/1.0/changelog".to_string());
                schema
            };
            let schema = serde_json::to_string_pretty(&schema)?;

            match destination {
                Some(path) => std::fs::write(path, &schema)?,
                None => print!("{}", &schema),
            };

            Ok(())
        }

        Command::Add {
            change_type,
            description,
        } => {
            let changelog_file = changelog_file?;
            let format = Format::try_from(&changelog_file)?;

            let mut changelog = Changelog::from_path(&changelog_file)?;

            match change_type {
                ChangeType::Added => changelog.unreleased.push_added(description),
                ChangeType::Changed => changelog.unreleased.push_changed(description),
                ChangeType::Deprecated => changelog.unreleased.push_deprecated(description),
                ChangeType::Removed => changelog.unreleased.push_removed(description),
                ChangeType::Fixed => changelog.unreleased.push_fixed(description),
                ChangeType::Security => changelog.unreleased.push_security(description),
            };

            std::fs::write(&changelog_file, format.to_string(&changelog)?)?;
            eprintln!("Added change to {}", &changelog_file.display());

            Ok(())
        }

        Command::Release {
            tag,
            date,
            version,
            description,
        } => {
            let changelog_file = changelog_file?;
            let format = Format::try_from(&changelog_file)?;
            let mut changelog = Changelog::from_path(&changelog_file)?;

            let date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
            let tag = tag.unwrap_or(version.clone());

            if changelog.versions.iter().any(|v| v.version == version) {
                bail!("Version {} already exists!", version);
            }

            let changes = changelog.unreleased;
            changelog.unreleased = Default::default();
            changelog.versions.insert(
                0,
                Version {
                    version,
                    tag,
                    date,
                    description,
                    changes,
                    ..Default::default()
                },
            );

            std::fs::write(&changelog_file, format.to_string(&changelog)?)?;

            Ok(())
        }

        Command::Yank { version, reason } => {
            let changelog_file = changelog_file?;
            let format = Format::try_from(&changelog_file)?;
            let mut changelog = Changelog::from_path(&changelog_file)?;

            let mut success = false;
            for released_version in &mut changelog.versions {
                if released_version.version == version {
                    released_version.yanked = Some(reason.clone());
                    success = true;
                }
            }

            if !success {
                eprintln!("Currently released versions:");
                for released_version in &changelog.versions {
                    eprintln!("  {}", released_version.version);
                }
                bail!("Could not find version {} to yank", version);
            }

            std::fs::write(&changelog_file, format.to_string(&changelog)?)?;

            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use std::process::Command;

    use assert_cmd::prelude::*;
    use assert_fs::{NamedTempFile, prelude::*};
    use predicates::prelude::*;
    use rstest::*;

    use super::Format;
    use changelog_md::{Changelog, Changes};

    fn predicate_is_yaml<Type: serde::de::DeserializeOwned>()
    -> predicates::function::FnPredicate<impl Fn(&str) -> bool, str> {
        predicate::function(|contents: &str| serde_yml::from_str::<Type>(&contents).is_ok())
    }

    fn predicate_is_toml<Type: serde::de::DeserializeOwned>()
    -> predicates::function::FnPredicate<impl Fn(&str) -> bool, str> {
        predicate::function(|contents: &str| toml::from_str::<Type>(&contents).is_ok())
    }

    fn predicate_is_json<Type: serde::de::DeserializeOwned>()
    -> predicates::function::FnPredicate<impl Fn(&str) -> bool, str> {
        predicate::function(|contents: &str| serde_json::from_str::<Type>(&contents).is_ok())
    }

    #[rstest]
    pub fn init_changelog(
        #[values(Format::Yaml, Format::Toml, Format::Json)] format: Format,
    ) -> anyhow::Result<()> {
        let tempdir = assert_fs::TempDir::new()?;

        let mut cmd = Command::cargo_bin("changelog-md")?;

        cmd.current_dir(&tempdir)
            .arg("init")
            .args(["--format", format.extension()])
            .assert()
            .success();

        let child = tempdir.child(format!("CHANGELOG.{}", format.extension()));

        child.assert(predicate::path::is_file());

        Changelog::from_path(child.path())?;

        Ok(())
    }

    /// Validate reading changelogs, by reading this repositories changelogs
    #[rstest]
    pub fn test_validate(
        #[values(Format::Yaml, Format::Toml, Format::Json)] format: Format,
    ) -> anyhow::Result<()> {
        let mut cmd = Command::cargo_bin("changelog-md")?;

        cmd.args(["--changelog", &format!("CHANGELOG.{}", format.extension())])
            .arg("validate")
            .assert()
            .success();

        Ok(())
    }

    #[rstest]
    pub fn test_render() -> anyhow::Result<()> {
        let mut cmd = Command::cargo_bin("changelog-md")?;
        let tmpfile = assert_fs::NamedTempFile::new("CHANGELOG.md")?;

        cmd.args(["--changelog", "CHANGELOG.yml"])
            .arg("render")
            .arg(tmpfile.path())
            .assert()
            .success();

        Ok(())
    }

    #[rstest]
    pub fn test_convert() -> anyhow::Result<()> {
        let tmpdir = assert_fs::TempDir::new()?;
        let yml = tmpdir.child("CHANGELOG.yml");
        let json = tmpdir.child("CHANGELOG.json");
        let toml = tmpdir.child("CHANGELOG.toml");

        Command::cargo_bin("changelog-md")?
            .current_dir(&tmpdir)
            .arg("init")
            .assert()
            .success();

        yml.assert(predicate::path::is_file())
            .assert(predicate_is_yaml::<Changelog>());

        Command::cargo_bin("changelog-md")?
            .current_dir(&tmpdir)
            .args(["--changelog", "CHANGELOG.yml"])
            .arg("convert")
            .args(["--format", "toml"])
            .assert()
            .success();
        toml.assert(predicate::path::is_file())
            .assert(predicate_is_toml::<Changelog>());

        Command::cargo_bin("changelog-md")?
            .current_dir(&tmpdir)
            .args(["--changelog", "CHANGELOG.yml"])
            .arg("convert")
            .args(["--format", "json"])
            .assert()
            .success();
        json.assert(predicate::path::is_file())
            .assert(predicate_is_json::<Changelog>());

        Ok(())
    }

    #[rstest]
    fn test_schema() -> anyhow::Result<()> {
        Command::cargo_bin("changelog-md")?
            .arg("schema")
            .assert()
            .success()
            .stdout(predicate_is_json::<schemars::schema::RootSchema>());

        let tmpfile = assert_fs::NamedTempFile::new("CHANGELOG.schema.json")?;

        Command::cargo_bin("changelog-md")?
            .arg("schema")
            .arg(&tmpfile.path())
            .assert()
            .success();

        tmpfile
            .assert(predicate::path::is_file())
            .assert(predicate_is_json::<schemars::schema::RootSchema>());

        Ok(())
    }

    #[rstest]
    fn test_add() -> anyhow::Result<()> {
        let tmpfile = NamedTempFile::new("CHANGELOG.yml")?;

        Command::cargo_bin("changelog-md")?
            .arg("--changelog")
            .arg(&tmpfile.path())
            .arg("init")
            .assert()
            .success();

        tmpfile.assert(predicate_is_yaml::<Changelog>());

        Command::cargo_bin("changelog-md")?
            .arg("--changelog")
            .arg(&tmpfile.path())
            .arg("add")
            .arg("changed")
            .arg("testing adding a new change")
            .assert()
            .success();

        tmpfile
            .assert(predicate_is_yaml::<Changelog>())
            .assert(predicate::function(|contents: &str| {
                let changelog = Changelog::from_yaml(contents).unwrap();

                changelog
                    .unreleased
                    .changed
                    .contains(&"testing adding a new change".to_string())
            }));

        Ok(())
    }

    #[rstest]
    fn test_release() -> anyhow::Result<()> {
        let tmpfile = NamedTempFile::new("CHANGELOG.yml")?;
        let changelog = Changelog {
            unreleased: Changes {
                changed: vec!["Testing releases".to_string()],
                ..Default::default()
            },
            versions: vec![],
            ..Default::default()
        };
        tmpfile.write_str(&changelog.to_yaml()?)?;

        Command::cargo_bin("changelog-md")?
            .arg("--changelog")
            .arg(&tmpfile.path())
            .arg("release")
            .args(["--tag", "v1.2.3"])
            .args(["--date", "2025-01-01"])
            .arg("1.2.3")
            .arg("some description")
            .assert()
            .success();

        tmpfile.assert(predicate::function(|contents: &str| {
            let changelog = Changelog::from_yaml(&contents).expect("Failed to parse");
            let version = changelog.versions.first().expect("Did not find a version");

            changelog.unreleased.changed.is_empty()
                && changelog.versions.len() == 1
                && version.version == "1.2.3"
                && version.tag == "v1.2.3"
                && version.date == "2025-01-01"
                && version.description == Some("some description".to_string())
                && version.changes
                    == Changes {
                        changed: vec!["Testing releases".to_string()],
                        ..Default::default()
                    }
        }));

        Ok(())
    }
}
