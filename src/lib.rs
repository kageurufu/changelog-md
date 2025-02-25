use anyhow::anyhow;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{KeyValueMap, serde_as};

/// A user-friendly format for writing Changelogs in a
/// verifiable and more git-friendly format
#[serde_as]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Changelog {
    /// # The heading
    pub title: String,
    /// A description of your project.
    /// It's recommended to note whether you follow semantic versioning
    pub description: String,
    /// Your source repository link
    pub repository: String,
    /// Currently unreleased changes
    pub unreleased: Changes,
    /// Releases
    #[serde_as(as = "KeyValueMap<_>")]
    pub versions: Vec<Version>,
}

/// A released version
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Version {
    /// The version name
    #[serde(rename = "$key$")]
    pub version: String,
    pub tag: String,
    /// Date the version was released as an ISO Date String
    #[schemars(regex(pattern = r"^\d{4}-[01]\d-[0-3]\d$"))]
    pub date: String,
    /// Optional Markdown description of this version
    pub description: Option<String>,
    /// If a version was yanked, the reason why
    #[serde(default)]
    pub yanked: Option<String>,
    #[serde(flatten)]
    pub changes: Changes,
}

/// Any changes made in this version
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Changes {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub added: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deprecated: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixed: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<String>,
}

impl Changes {
    pub fn push_added(&mut self, change: String) {
        self.added.push(change)
    }
    pub fn push_changed(&mut self, change: String) {
        self.changed.push(change)
    }
    pub fn push_deprecated(&mut self, change: String) {
        self.deprecated.push(change)
    }
    pub fn push_fixed(&mut self, change: String) {
        self.fixed.push(change)
    }
    pub fn push_removed(&mut self, change: String) {
        self.removed.push(change)
    }
    pub fn push_security(&mut self, change: String) {
        self.security.push(change)
    }

    fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.changed.is_empty()
            && self.deprecated.is_empty()
            && self.fixed.is_empty()
            && self.removed.is_empty()
            && self.security.is_empty()
    }

    // Helper to write a block of changes
    fn write_changes_if_exist(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        title: &str,
        changes: &Vec<String>,
    ) -> std::fmt::Result {
        if !changes.is_empty() {
            writeln!(f, "### {}", title)?;
            for change in changes {
                writeln!(f, "- {}", change)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Changelog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "# {}", self.title)?;
        writeln!(f)?;
        writeln!(f, "{}", self.description)?;
        if !self.description.ends_with("\n") {
            writeln!(f)?;
        }
        if !self.unreleased.is_empty() {
            writeln!(f, "## [Unreleased]")?;
            writeln!(f)?;
            writeln!(f, "{}", self.unreleased)?;
        }

        for version in &self.versions {
            write!(f, "{}", version)?;
        }

        match &self.versions[..] {
            // We haven't released a version, just link all commits
            [] => writeln!(f, "[unreleased] {}/commits/", self.repository)?,

            versions @ [.., last] => {
                writeln!(
                    f,
                    "[unreleased] {}/compare/{}...HEAD",
                    self.repository, versions[0].tag
                )?;
                for idx in 0..(versions.len() - 1) {
                    writeln!(
                        f,
                        "[{}] {}/compare/{}..{}",
                        versions[idx].version,
                        self.repository,
                        versions[idx + 1].tag,
                        versions[idx].tag,
                    )?;
                }
                // The initial version is a commit url
                writeln!(
                    f,
                    "[{}] {}/commits/{}",
                    last.version, self.repository, last.tag
                )?;
            }
        };

        Ok(())
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "## {} - {}", self.version, self.date)?;
        if let Some(reason) = &self.yanked {
            write!(f, " [YANKED] {}", reason)?;
        }
        writeln!(f)?;
        writeln!(f)?;
        if let Some(desc) = &self.description {
            writeln!(f, "{}", desc)?;
        }
        if !self.changes.is_empty() {
            writeln!(f, "{}", self.changes)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for Changes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_changes_if_exist(f, "Added", &self.added)?;
        self.write_changes_if_exist(f, "Changed", &self.changed)?;
        self.write_changes_if_exist(f, "Deprecated", &self.deprecated)?;
        self.write_changes_if_exist(f, "Removed", &self.removed)?;
        self.write_changes_if_exist(f, "Fixed", &self.fixed)?;
        self.write_changes_if_exist(f, "Security", &self.security)?;

        Ok(())
    }
}

impl Changelog {
    pub fn from_path(path: impl Into<std::path::PathBuf>) -> anyhow::Result<Changelog> {
        let path = path.into();

        if !path.exists() {
            return Err(anyhow!("no such file {}", path.display()));
        }

        match path.extension().map(|e| e.to_ascii_lowercase()) {
            Some(e) if e == "yml" || e == "yaml" => {
                let s = &std::fs::read_to_string(path)?;
                Self::from_yaml(s)
            }
            Some(e) if e == "toml" => {
                let s = &std::fs::read_to_string(path)?;
                Self::from_toml(s)
            }
            Some(e) if e == "json" => {
                let s = &std::fs::read_to_string(path)?;
                Self::from_json(s)
            }
            Some(e) => Err(anyhow!("Invalid file extension {}", e.to_string_lossy())),
            None => Err(anyhow!(
                "Unable to read {} without an extension",
                path.display()
            )),
        }
    }

    pub fn from_yaml(s: &str) -> anyhow::Result<Changelog> {
        let de = serde_yml::Deserializer::from_str(s);
        Ok(serde_path_to_error::deserialize(de)?)
    }

    pub fn from_json(s: &str) -> anyhow::Result<Changelog> {
        let mut de = serde_json::Deserializer::from_str(s);
        Ok(serde_path_to_error::deserialize(&mut de)?)
    }

    pub fn from_toml(s: &str) -> anyhow::Result<Changelog> {
        let de = toml::Deserializer::new(s);
        Ok(serde_path_to_error::deserialize(de)?)
    }

    pub fn to_yaml(&self) -> anyhow::Result<String> {
        Ok(serde_yml::to_string(&self)?)
    }

    pub fn to_toml(&self) -> anyhow::Result<String> {
        Ok(toml::to_string_pretty(&self)?)
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(&self)? + "\n")
    }
}

impl Default for Changelog {
    fn default() -> Self {
        Self {
            title: "Changelog".into(),
            description: r#"All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
"#
            .into(),
            // TODO: Parse .git/config '[remote "origin"].url'
            repository: "https://github.com/me/my-swanky-project".into(),
            unreleased: Default::default(),
            versions: vec![Version {
                version: "0.0.1".into(),
                tag: "0.0.1".into(),
                date: "1970-01-01".into(),
                ..Default::default()
            }],
        }
    }
}
