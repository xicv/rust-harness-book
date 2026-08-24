use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::string::FromUtf8Error;

// ANCHOR: ch01_toolchain_report
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ToolchainReport {
    pinned_rust: String,
    active_rustc: String,
    active_cargo: String,
    edition: String,
    resolver: String,
    lockfile_present: bool,
}

impl ToolchainReport {
    pub(crate) fn collect() -> Result<Self, DoctorError> {
        let root = workspace_root()?;
        Self::collect_from(&root)
    }

    fn collect_from(root: &Path) -> Result<Self, DoctorError> {
        let toolchain = read_source(root.join("rust-toolchain.toml"))?;
        let manifest = read_source(root.join("Cargo.toml"))?;

        Ok(Self {
            pinned_rust: quoted_value(&toolchain, "channel")?.to_owned(),
            active_rustc: command_version("rustc")?,
            active_cargo: command_version("cargo")?,
            edition: quoted_value(&manifest, "edition")?.to_owned(),
            resolver: quoted_value(&manifest, "resolver")?.to_owned(),
            lockfile_present: root.join("Cargo.lock").is_file(),
        })
    }

    #[must_use]
    pub(crate) fn is_match(&self) -> bool {
        self.active_rustc == self.pinned_rust
            && self.active_cargo == self.pinned_rust
            && self.lockfile_present
    }
}

impl fmt::Display for ToolchainReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_match() { "ok" } else { "mismatch" };
        let lockfile = if self.lockfile_present {
            "present"
        } else {
            "missing"
        };
        let pinned_rust = &self.pinned_rust;
        let active_rustc = &self.active_rustc;
        let active_cargo = &self.active_cargo;
        let edition = &self.edition;
        let resolver = &self.resolver;

        writeln!(formatter, "toolchain/status {status}")?;
        writeln!(formatter, "rust/pinned {pinned_rust}")?;
        writeln!(formatter, "rustc/active {active_rustc}")?;
        writeln!(formatter, "cargo/active {active_cargo}")?;
        writeln!(formatter, "edition {edition}")?;
        writeln!(formatter, "resolver {resolver}")?;
        writeln!(formatter, "lockfile {lockfile}")
    }
}
// ANCHOR_END: ch01_toolchain_report

#[derive(Debug)]
pub(crate) enum DoctorError {
    CurrentDirectory(std::io::Error),
    WorkspaceNotFound,
    ReadSource {
        path: PathBuf,
        source: std::io::Error,
    },
    MissingValue(&'static str),
    CommandIo {
        program: &'static str,
        source: std::io::Error,
    },
    CommandFailed(&'static str),
    InvalidUtf8 {
        program: &'static str,
        source: FromUtf8Error,
    },
    InvalidVersionOutput(&'static str),
}

impl fmt::Display for DoctorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentDirectory(_) => {
                formatter.write_str("could not read the current directory")
            }
            Self::WorkspaceNotFound => {
                formatter.write_str("could not find the rust-harness-book workspace")
            }
            Self::ReadSource { path, .. } => {
                write!(formatter, "could not read {}", path.display())
            }
            Self::MissingValue(key) => write!(formatter, "missing source-controlled value: {key}"),
            Self::CommandIo { program, .. } => {
                write!(formatter, "could not run {program} --version")
            }
            Self::CommandFailed(program) => {
                write!(formatter, "{program} --version returned a failure status")
            }
            Self::InvalidUtf8 { program, .. } => {
                write!(formatter, "{program} --version returned non-UTF-8 output")
            }
            Self::InvalidVersionOutput(program) => {
                write!(
                    formatter,
                    "{program} --version returned an unexpected format"
                )
            }
        }
    }
}

impl Error for DoctorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CurrentDirectory(source)
            | Self::ReadSource { source, .. }
            | Self::CommandIo { source, .. } => Some(source),
            Self::InvalidUtf8 { source, .. } => Some(source),
            Self::WorkspaceNotFound
            | Self::MissingValue(_)
            | Self::CommandFailed(_)
            | Self::InvalidVersionOutput(_) => None,
        }
    }
}

fn workspace_root() -> Result<PathBuf, DoctorError> {
    let mut directory = env::current_dir().map_err(DoctorError::CurrentDirectory)?;

    loop {
        if directory.join("rust-toolchain.toml").is_file() && directory.join("Cargo.toml").is_file()
        {
            return Ok(directory);
        }
        if !directory.pop() {
            return Err(DoctorError::WorkspaceNotFound);
        }
    }
}

fn read_source(path: PathBuf) -> Result<String, DoctorError> {
    fs::read_to_string(&path).map_err(|source| DoctorError::ReadSource { path, source })
}

fn quoted_value<'a>(text: &'a str, key: &'static str) -> Result<&'a str, DoctorError> {
    text.lines()
        .filter_map(|line| line.split('#').next())
        .filter_map(|line| line.split_once('='))
        .find_map(|(candidate, value)| {
            if candidate.trim() == key {
                Some(value.trim().trim_matches('"'))
            } else {
                None
            }
        })
        .filter(|value| !value.is_empty())
        .ok_or(DoctorError::MissingValue(key))
}

fn command_version(program: &'static str) -> Result<String, DoctorError> {
    let output = Command::new(program)
        .arg("--version")
        .output()
        .map_err(|source| DoctorError::CommandIo { program, source })?;

    if !output.status.success() {
        return Err(DoctorError::CommandFailed(program));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|source| DoctorError::InvalidUtf8 { program, source })?;

    stdout
        .split_whitespace()
        .nth(1)
        .map(str::to_owned)
        .ok_or(DoctorError::InvalidVersionOutput(program))
}
