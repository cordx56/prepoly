use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

/// Top-level structure of `package.toml`.
#[derive(Deserialize)]
pub struct Manifest {
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: BTreeMap<String, Dependency>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct PackageInfo {
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub license: String,
}

/// A single dependency entry:
/// `"name" = { git = "https://...", hash = "..." }`
#[derive(Deserialize)]
pub struct Dependency {
    pub git: String,
    pub hash: String,
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Manifest, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
        toml::from_str(&content).map_err(|e| format!("cannot parse `{}`: {e}", path.display()))
    }
}
