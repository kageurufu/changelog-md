use anyhow::anyhow;
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

    /// Render a CHANGELOG to Markdown
    Render {
        /// Destination path
        destination: Option<std::path::PathBuf>,
    },
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
                let seed = Changelog::default();
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
    }
}
