use anyhow::anyhow;
use changelog_md::Changelog;

use clap::{Parser, Subcommand, ValueEnum};
use schemars::schema_for;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate an initial CHANGELOG.yml
    Init {
        #[clap(short, long, default_value = "Format::Yaml")]
        format: Format,
    },

    /// Convert a CHANGELOG source to another format
    Convert {
        #[clap(long, default_value = "false")]
        force: bool,
        #[clap(short, long, default_value = "yaml")]
        format: Format,
        source: Option<std::path::PathBuf>,
    },

    /// Validate a CHANGELOG
    Validate {
        /// CHANGELOG source file (yaml, json, or toml)
        source: Option<std::path::PathBuf>,
    },

    /// Get the CHANGELOG schema
    Schema {
        /// Destination to write the changelog
        destination: Option<std::path::PathBuf>,
    },

    /// Render a CHANGELOG to Markdown
    Render {
        /// CHANGELOG source file (yaml, json, or toml)
        source: Option<std::path::PathBuf>,
        /// Destination path
        destination: Option<std::path::PathBuf>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum Format {
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
            Format::Yaml => Ok(serde_yaml::to_string(&seed)?),
            Format::Toml => Ok(toml::to_string(&seed)?),
            Format::Json => Ok(serde_json::to_string_pretty(&seed)? + "\n"),
        }
    }
}

fn autodetect_source() -> Option<std::path::PathBuf> {
    for entry in std::fs::read_dir(".").unwrap() {
        let path = entry.unwrap().path();
        if let Some(file_stem) = path.file_stem() {
            if let Some(ext) = path.extension() {
                if file_stem.to_ascii_lowercase() == "changelog"
                    && (ext == "yml" || ext == "yaml" || ext == "toml" || ext == "json")
                {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init { format } => {
            let filename: std::path::PathBuf = format!("CHANGELOG.{}", format.extension()).into();

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

        Command::Convert {
            format,
            force,
            source,
        } => match source.or_else(|| autodetect_source()) {
            None => Err(anyhow!("Unable to find a CHANGELOG source file")),
            Some(source) => {
                let destination = source.with_extension(format.extension());
                if destination.exists() && !force {
                    Err(anyhow!("{} already exists", destination.display()))
                } else {
                    let changelog = Changelog::from_path(&source)?;
                    let changelog = format.to_string(&changelog)?;
                    eprintln!(
                        "Converting {} to {}",
                        source.display(),
                        destination.display()
                    );
                    std::fs::write(destination, changelog)?;

                    Ok(())
                }
            }
        },

        Command::Render {
            source,
            destination,
        } => match source.or_else(|| autodetect_source()) {
            None => Err(anyhow!("Unable to find a CHANGELOG source file")),
            Some(source) => {
                let changelog = Changelog::from_path(&source)?;

                let path = destination.unwrap_or_else(|| source.with_extension("md"));
                eprintln!("Rendering {} to {}", source.display(), path.display());
                Ok(std::fs::write(path, format!("{}", &changelog))?)
            }
        },

        Command::Validate { source } => match source.or_else(|| autodetect_source()) {
            None => Err(anyhow!("Unable to find a CHANGELOG source file")),
            Some(source) => {
                Changelog::from_path(&source)?;
                println!("No issues found");
                Ok(())
            }
        },

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
    }
}
