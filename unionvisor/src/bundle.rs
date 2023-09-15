use std::{
    ffi::OsString,
    fs, io,
    path::PathBuf,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, field::display as as_display};

/// Bundles should have the following structure on the filesystem:
///
/// ```text
/// bundle
/// ├── meta.json
/// └── versions
///     └── v0.5.0
///         └── uniond
/// ```
///
/// Or, more generally:
///
/// ```text
/// bundle
/// ├── meta.json
/// └── META.VERSIONS_PATH
///     └── BINARY_VERSION_IDENTIFIER
///         └── META.BINARY_NAME
/// ```
///
/// Bundle meta information is defined in meta.json, which gets deserialized to [`BundleMeta`]
#[derive(Clone, Debug)]
pub struct Bundle {
    /// The path of the bundle
    pub path: PathBuf,
    /// The deserialized meta info from `bundle/meta.json`
    meta: BundleMeta,
}

/// Version paths that have not been validated.
/// The inner [`PathBuf`] is not public as it should not be accessed.
pub struct UnvalidatedVersionPath(PathBuf);

impl UnvalidatedVersionPath {
    /// Constructs a new [`UnvalidatedVersionPath`] based on any `PathBuf`]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self(path)
    }
}

/// Version paths that have been validated.
/// This means that the binary at this path produces valid output when called with --help
pub struct ValidVersionPath(pub PathBuf);

#[derive(Debug, Error)]
pub enum ValidateVersionPathError {
    #[error("version was not found in bundle: {0}")]
    NotInBundle(PathBuf, #[source] io::Error),
    #[error("permision denied when executing version in bundle: {0}")]
    PermisionDenied(PathBuf, #[source] io::Error),
    #[error("other IO error")]
    OtherIO(#[source] io::Error),
}

impl UnvalidatedVersionPath {
    /// Validates a [`UnvalidatedVersionPath`], turning it into a [`ValidVersionPath`] if validation is successful
    pub fn validate(&self) -> Result<ValidVersionPath, ValidateVersionPathError> {
        debug!(
            "testing if binary {} is available by calling --help",
            as_display(self.0.display())
        );
        let mut child = Command::new(&self.0)
            .arg("--help")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .spawn()?;
        debug!(target: "unionvisor", "killing test call of {}", as_display(self.0.display()));

        if let Err(err) = child.kill() {
            match err.kind() {
                io::ErrorKind::NotFound => {
                    return Err(ValidateVersionPathError::NotInBundle(self.0, err))
                }
                io::ErrorKind::PermissionDenied => {
                    return Err(BinaryAvailability::PermissionDenied(self.0, err))
                }
                _ => return Err(ValidateVersionPathError::OtherIO(err)),
            }
        }
        Ok(ValidVersionPath(self.0.clone()))
    }
}

/// Bundle meta info found in `bundle/meta.json`
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct BundleMeta {
    /// The name of the binary in `bundle/bins/$VERSION/`
    binary_name: String,
    /// The fallback version directory in `bundle/bins/`
    fallback_version: String,
    /// The directory containing a directory for each version
    versions_directory: PathBuf,
}

pub enum BinaryAvailability {
    NotFound,
    PermissionDenied,
    Ok,
}

impl Bundle {
    /// Constructs a new [`Bundle`] based on a path.
    /// Will read `bundle/meta.json` and error if invalid.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path: PathBuf = path.into();

        // Read `bundle/meta.json` and deserialize into `BundleMeta`
        let meta = path.join("meta.json");
        let meta = fs::read_to_string(meta).map_err(|_| {
            eyre!("Can't find meta.json in bundle. Please make sure it exists at bundle/meta.json")
        })?;
        let meta = serde_json::from_str(&meta).map_err(|_| {
            eyre!(
                "Can't deserialize bundle/meta.json. Please ensure that it adheres to the scheme."
            )
        })?;

        let bundle = Bundle { path, meta };

        Ok(bundle)
    }

    /// Obtains the path to the binary within the bundle with version `version`.
    pub fn path_to(&self, version: impl Into<OsString>) -> UnvalidatedVersionPath {
        let version = version.into();
        UnvalidatedVersionPath::new(
            self.path
                .join(&self.meta.versions_directory)
                .join(version)
                .join(&self.meta.binary_name),
        )
    }

    /// Construct the path to the fallback verison, based on the [`BundleMeta`]
    pub fn fallback_path(&self) -> Result<ValidVersionPath> {
        let fallback_version = &self.meta.fallback_version.clone();
        self.path_to(fallback_version).validate()
    }
}
