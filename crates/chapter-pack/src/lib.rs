#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use sha2::{Digest, Sha256};
use toml::Value;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

static NEXT_VERIFICATION_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub struct BuildRequest {
    project_root: PathBuf,
    manifest_path: PathBuf,
    output_directory: PathBuf,
}

impl BuildRequest {
    pub fn new(
        project_root: impl Into<PathBuf>,
        manifest_path: impl Into<PathBuf>,
        output_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            project_root: project_root.into(),
            manifest_path: manifest_path.into(),
            output_directory: output_directory.into(),
        }
    }
}

#[derive(Debug)]
pub struct BuildReport {
    archive_path: PathBuf,
    checksum_path: PathBuf,
    source_commit: String,
    verified_command_count: usize,
}

impl BuildReport {
    pub fn archive_path(&self) -> &Path {
        &self.archive_path
    }

    pub fn checksum_path(&self) -> &Path {
        &self.checksum_path
    }

    pub fn source_commit(&self) -> &str {
        &self.source_commit
    }

    pub fn verified_command_count(&self) -> usize {
        self.verified_command_count
    }
}

#[derive(Debug)]
pub struct PackError {
    message: String,
}

impl PackError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn with_context(context: &str, error: impl fmt::Display) -> Self {
        Self::new(format!("{context}: {error}"))
    }
}

impl fmt::Display for PackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl Error for PackError {}

pub fn build_and_verify(request: &BuildRequest) -> Result<BuildReport, PackError> {
    let project_root = canonical_directory(&request.project_root, "project root")?;
    let manifest_path = canonical_file(&request.manifest_path, "pack manifest")?;
    require_within(&manifest_path, &project_root, "pack manifest")?;
    let manifest_text = fs::read_to_string(&manifest_path)
        .map_err(|error| PackError::with_context("could not read pack manifest", error))?;
    let manifest = Manifest::parse(&manifest_text)?;

    verify_source_commit(&project_root, &manifest.source_commit)?;
    let files = collect_files(&project_root, &manifest)?;
    fs::create_dir_all(&request.output_directory)
        .map_err(|error| PackError::with_context("could not create output directory", error))?;
    let output_directory = canonical_directory(&request.output_directory, "output directory")?;
    let archive_path = output_directory.join(&manifest.archive_name);
    write_archive(&archive_path, &manifest.root_directory, &files)?;

    let archive_bytes = fs::read(&archive_path)
        .map_err(|error| PackError::with_context("could not read generated archive", error))?;
    let checksum_path = output_directory.join(format!("{}.sha256", manifest.archive_name));
    fs::write(&checksum_path, format!("{}\n", sha256_hex(&archive_bytes)))
        .map_err(|error| PackError::with_context("could not write archive checksum", error))?;

    verify_archive(&archive_path, &manifest, &files, &output_directory)?;

    Ok(BuildReport {
        archive_path,
        checksum_path,
        source_commit: manifest.source_commit,
        verified_command_count: manifest.commands.len(),
    })
}

#[derive(Debug)]
struct Manifest {
    source_commit: String,
    archive_name: String,
    root_directory: String,
    source_paths: Vec<String>,
    generated_files: Vec<GeneratedFile>,
    commands: Vec<VerificationCommand>,
}

impl Manifest {
    fn parse(source: &str) -> Result<Self, PackError> {
        let value = toml::from_str::<Value>(source)
            .map_err(|error| PackError::with_context("invalid pack manifest TOML", error))?;
        let table = value
            .as_table()
            .ok_or_else(|| PackError::new("pack manifest must be a TOML table"))?;
        require_only_keys(
            table.keys().map(String::as_str),
            &[
                "schema",
                "chapter",
                "source_commit",
                "archive_name",
                "root_directory",
                "source_paths",
                "generated_files",
                "commands",
            ],
            "pack manifest",
        )?;
        if required_integer(table, "schema")? != 1 {
            return Err(PackError::new("pack manifest schema must be 1"));
        }
        validate_identifier(required_string(table, "chapter")?, "chapter")?;
        let source_commit = required_string(table, "source_commit")?.to_owned();
        if source_commit.len() != 40
            || !source_commit
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(PackError::new(
                "source_commit must be a full lowercase hexadecimal Git object ID",
            ));
        }
        let archive_name = required_string(table, "archive_name")?.to_owned();
        validate_file_name(&archive_name, "archive_name")?;
        if !archive_name.ends_with(".zip") {
            return Err(PackError::new("archive_name must end in .zip"));
        }
        let root_directory = required_string(table, "root_directory")?.to_owned();
        validate_file_name(&root_directory, "root_directory")?;

        let source_paths = required_string_array(table, "source_paths")?;
        if source_paths.is_empty() {
            return Err(PackError::new("source_paths must not be empty"));
        }
        require_unique_paths(&source_paths, "source_paths")?;
        for path in &source_paths {
            validate_relative_path(path, "source path")?;
        }

        let generated_files = parse_generated_files(table)?;
        if generated_files.is_empty() {
            return Err(PackError::new("generated_files must not be empty"));
        }
        let generated_destinations = generated_files
            .iter()
            .map(|file| file.destination.clone())
            .collect::<Vec<_>>();
        require_unique_paths(&generated_destinations, "generated file destinations")?;

        let commands = parse_commands(table)?;
        if commands.is_empty() {
            return Err(PackError::new("commands must not be empty"));
        }

        Ok(Self {
            source_commit,
            archive_name,
            root_directory,
            source_paths,
            generated_files,
            commands,
        })
    }
}

#[derive(Debug)]
struct GeneratedFile {
    source: String,
    destination: String,
}

#[derive(Debug)]
struct VerificationCommand {
    argv: Vec<String>,
    expected_stdout: Option<String>,
}

fn parse_generated_files(
    manifest: &toml::map::Map<String, Value>,
) -> Result<Vec<GeneratedFile>, PackError> {
    let entries = required_array(manifest, "generated_files")?;
    entries
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let table = value.as_table().ok_or_else(|| {
                PackError::new(format!("generated_files[{index}] must be a table"))
            })?;
            require_only_keys(
                table.keys().map(String::as_str),
                &["source", "destination"],
                &format!("generated_files[{index}]"),
            )?;
            let source = required_string(table, "source")?.to_owned();
            let destination = required_string(table, "destination")?.to_owned();
            validate_relative_path(&source, "generated file source")?;
            validate_relative_path(&destination, "generated file destination")?;
            Ok(GeneratedFile {
                source,
                destination,
            })
        })
        .collect()
}

fn parse_commands(
    manifest: &toml::map::Map<String, Value>,
) -> Result<Vec<VerificationCommand>, PackError> {
    let entries = required_array(manifest, "commands")?;
    entries
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let table = value
                .as_table()
                .ok_or_else(|| PackError::new(format!("commands[{index}] must be a table")))?;
            require_only_keys(
                table.keys().map(String::as_str),
                &["argv", "expected_stdout"],
                &format!("commands[{index}]"),
            )?;
            let argv = required_string_array(table, "argv")?;
            if argv.first().map(String::as_str) != Some("cargo") {
                return Err(PackError::new(format!(
                    "commands[{index}] must invoke cargo directly"
                )));
            }
            if argv.iter().any(String::is_empty) {
                return Err(PackError::new(format!(
                    "commands[{index}] arguments must not be empty"
                )));
            }
            validate_command_arguments(&argv, index)?;
            let expected_stdout = table
                .get("expected_stdout")
                .map(|value| {
                    value
                        .as_str()
                        .ok_or_else(|| PackError::new("expected_stdout must be a string"))
                        .map(str::to_owned)
                })
                .transpose()?;
            if let Some(path) = &expected_stdout {
                validate_relative_path(path, "expected_stdout")?;
            }
            Ok(VerificationCommand {
                argv,
                expected_stdout,
            })
        })
        .collect()
}

fn validate_command_arguments(argv: &[String], index: usize) -> Result<(), PackError> {
    let is_workspace_test = argv
        == [
            "cargo".to_owned(),
            "test".to_owned(),
            "--workspace".to_owned(),
            "--locked".to_owned(),
        ];
    let is_package_run = argv.len() >= 7
        && argv[1] == "run"
        && argv[2] == "-p"
        && validate_package_name(&argv[3])
        && argv[4] == "--locked"
        && argv[5] == "--"
        && argv[6..].iter().all(|argument| {
            !argument.is_empty() && !argument.contains('\0') && !Path::new(argument).is_absolute()
        });
    if !is_workspace_test && !is_package_run {
        return Err(PackError::new(format!(
            "commands[{index}] is not a supported offline form"
        )));
    }
    Ok(())
}

fn validate_package_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn collect_files(
    project_root: &Path,
    manifest: &Manifest,
) -> Result<BTreeMap<String, Vec<u8>>, PackError> {
    let mut files = BTreeMap::new();
    for path in &manifest.source_paths {
        require_regular_git_blob(project_root, &manifest.source_commit, path)?;
        let object = format!("{}:{path}", manifest.source_commit);
        let output = run_git(project_root, ["cat-file", "blob", &object])?;
        require_success(output.status.success(), "git cat-file", &output.stderr)?;
        insert_unique(&mut files, path, output.stdout)?;
    }
    for generated in &manifest.generated_files {
        let source_path = project_root.join(&generated.source);
        let canonical_source = canonical_file(&source_path, "generated file source")?;
        require_within(&canonical_source, project_root, "generated file source")?;
        let metadata = fs::symlink_metadata(&source_path)
            .map_err(|error| PackError::with_context("could not inspect generated file", error))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(PackError::new(format!(
                "generated file source must be a regular file: {}",
                generated.source
            )));
        }
        let bytes = fs::read(&canonical_source)
            .map_err(|error| PackError::with_context("could not read generated file", error))?;
        insert_unique(&mut files, &generated.destination, bytes)?;
    }
    Ok(files)
}

fn insert_unique(
    files: &mut BTreeMap<String, Vec<u8>>,
    path: &str,
    bytes: Vec<u8>,
) -> Result<(), PackError> {
    if files.insert(path.to_owned(), bytes).is_some() {
        return Err(PackError::new(format!(
            "duplicate archive destination: {path}"
        )));
    }
    Ok(())
}

fn verify_source_commit(project_root: &Path, source_commit: &str) -> Result<(), PackError> {
    let commit_object = format!("{source_commit}^{{commit}}");
    let object = run_git(project_root, ["cat-file", "-e", &commit_object])?;
    require_success(
        object.status.success(),
        "source_commit is not a local Git commit",
        &object.stderr,
    )?;
    let ancestor = run_git(
        project_root,
        ["merge-base", "--is-ancestor", source_commit, "HEAD"],
    )?;
    require_success(
        ancestor.status.success(),
        "source_commit is not an ancestor of HEAD",
        &ancestor.stderr,
    )
}

fn require_regular_git_blob(
    project_root: &Path,
    source_commit: &str,
    path: &str,
) -> Result<(), PackError> {
    let output = run_git(project_root, ["ls-tree", "-z", source_commit, "--", path])?;
    require_success(output.status.success(), "git ls-tree", &output.stderr)?;
    let record = std::str::from_utf8(&output.stdout)
        .map_err(|error| PackError::with_context("git ls-tree returned non-UTF-8 data", error))?
        .strip_suffix('\0')
        .ok_or_else(|| PackError::new(format!("source path does not exist: {path}")))?;
    if record.contains('\0') {
        return Err(PackError::new(format!(
            "source path resolved to multiple Git entries: {path}"
        )));
    }
    let (header, returned_path) = record
        .split_once('\t')
        .ok_or_else(|| PackError::new("unexpected git ls-tree output"))?;
    let mut fields = header.split_whitespace();
    let mode = fields.next().unwrap_or_default();
    let object_type = fields.next().unwrap_or_default();
    if fields.next().is_none() || fields.next().is_some() {
        return Err(PackError::new("unexpected git ls-tree header"));
    }
    if returned_path != path || object_type != "blob" || !matches!(mode, "100644" | "100755") {
        return Err(PackError::new(format!(
            "source path must be an exact regular Git blob: {path}"
        )));
    }
    Ok(())
}

fn write_archive(
    archive_path: &Path,
    root_directory: &str,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<(), PackError> {
    let file = File::create(archive_path)
        .map_err(|error| PackError::with_context("could not create archive", error))?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o644);
    for (path, bytes) in files {
        writer
            .start_file(format!("{root_directory}/{path}"), options)
            .map_err(|error| PackError::with_context("could not start archive entry", error))?;
        writer
            .write_all(bytes)
            .map_err(|error| PackError::with_context("could not write archive entry", error))?;
    }
    writer
        .finish()
        .map_err(|error| PackError::with_context("could not finish archive", error))?;
    Ok(())
}

fn verify_archive(
    archive_path: &Path,
    manifest: &Manifest,
    files: &BTreeMap<String, Vec<u8>>,
    output_directory: &Path,
) -> Result<(), PackError> {
    let verification_directory = VerificationDirectory::new(output_directory)?;
    let archive_file = File::open(archive_path).map_err(|error| {
        PackError::with_context("could not open archive for verification", error)
    })?;
    let mut archive = ZipArchive::new(archive_file)
        .map_err(|error| PackError::with_context("could not parse generated archive", error))?;
    if archive.len() != files.len() {
        return Err(PackError::new(
            "archive entry count does not match manifest",
        ));
    }
    let expected_names = files
        .keys()
        .map(|path| format!("{}/{path}", manifest.root_directory))
        .collect::<BTreeSet<_>>();
    let mut extracted_names = BTreeSet::new();
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| PackError::with_context("could not read archive entry", error))?;
        let name = entry.name().to_owned();
        if !expected_names.contains(&name) || !extracted_names.insert(name.clone()) {
            return Err(PackError::new(format!(
                "archive contains an unexpected or duplicate entry: {name}"
            )));
        }
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| PackError::new(format!("unsafe archive entry path: {name}")))?
            .to_owned();
        if entry.is_dir() || entry.unix_mode() != Some(0o100644) {
            return Err(PackError::new(format!(
                "archive entry is not a regular read-only data file: {name}"
            )));
        }
        let destination = verification_directory.path().join(enclosed);
        let parent = destination
            .parent()
            .ok_or_else(|| PackError::new(format!("archive entry has no parent: {name}")))?;
        fs::create_dir_all(parent).map_err(|error| {
            PackError::with_context("could not create extraction directory", error)
        })?;
        let mut destination_file = File::create(&destination)
            .map_err(|error| PackError::with_context("could not create extracted file", error))?;
        std::io::copy(&mut entry, &mut destination_file)
            .map_err(|error| PackError::with_context("could not extract archive entry", error))?;
    }
    if extracted_names != expected_names {
        return Err(PackError::new("archive entries do not match manifest"));
    }

    let extracted_root = verification_directory.path().join(&manifest.root_directory);
    for (path, expected) in files {
        let actual = fs::read(extracted_root.join(path))
            .map_err(|error| PackError::with_context("could not re-read extracted file", error))?;
        if &actual != expected {
            return Err(PackError::new(format!(
                "extracted file does not match its source: {path}"
            )));
        }
    }
    for command in &manifest.commands {
        verify_command(command, &extracted_root)?;
    }
    Ok(())
}

fn verify_command(
    command: &VerificationCommand,
    working_directory: &Path,
) -> Result<(), PackError> {
    let executable = command
        .argv
        .first()
        .ok_or_else(|| PackError::new("verification command has no executable"))?;
    let output = Command::new(executable)
        .args(&command.argv[1..])
        .current_dir(working_directory)
        .env("CARGO_NET_OFFLINE", "true")
        .env("CARGO_TERM_COLOR", "never")
        .output()
        .map_err(|error| PackError::with_context("could not run verification command", error))?;
    require_success(
        output.status.success(),
        &format!("verification command failed: {}", command.argv.join(" ")),
        &output.stderr,
    )?;
    if let Some(expected_path) = &command.expected_stdout {
        let expected = fs::read(working_directory.join(expected_path)).map_err(|error| {
            PackError::with_context("could not read expected command output", error)
        })?;
        if output.stdout != expected {
            return Err(PackError::new(format!(
                "verification command stdout did not match {expected_path}: {}",
                command.argv.join(" ")
            )));
        }
    }
    Ok(())
}

struct VerificationDirectory {
    path: PathBuf,
}

impl VerificationDirectory {
    fn new(parent: &Path) -> Result<Self, PackError> {
        let sequence = NEXT_VERIFICATION_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(
            ".chapter-pack-verification-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).map_err(|error| {
            PackError::with_context("could not create verification directory", error)
        })?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for VerificationDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn required_array<'a>(
    table: &'a toml::map::Map<String, Value>,
    key: &str,
) -> Result<&'a Vec<Value>, PackError> {
    table
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| PackError::new(format!("{key} must be an array")))
}

fn required_string_array(
    table: &toml::map::Map<String, Value>,
    key: &str,
) -> Result<Vec<String>, PackError> {
    required_array(table, key)?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| PackError::new(format!("{key} entries must be strings")))
        })
        .collect()
}

fn required_string<'a>(
    table: &'a toml::map::Map<String, Value>,
    key: &str,
) -> Result<&'a str, PackError> {
    table
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| PackError::new(format!("{key} must be a string")))
}

fn required_integer(table: &toml::map::Map<String, Value>, key: &str) -> Result<i64, PackError> {
    table
        .get(key)
        .and_then(Value::as_integer)
        .ok_or_else(|| PackError::new(format!("{key} must be an integer")))
}

fn require_only_keys<'a>(
    keys: impl Iterator<Item = &'a str>,
    allowed: &[&str],
    context: &str,
) -> Result<(), PackError> {
    for key in keys {
        if !allowed.contains(&key) {
            return Err(PackError::new(format!(
                "unsupported key {key:?} in {context}"
            )));
        }
    }
    Ok(())
}

fn require_unique_paths(paths: &[String], context: &str) -> Result<(), PackError> {
    let mut unique = BTreeSet::new();
    for path in paths {
        if !unique.insert(path) {
            return Err(PackError::new(format!(
                "duplicate path in {context}: {path}"
            )));
        }
    }
    Ok(())
}

fn validate_identifier(value: &str, field: &str) -> Result<(), PackError> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    {
        return Err(PackError::new(format!(
            "{field} must contain only lowercase ASCII letters and digits"
        )));
    }
    Ok(())
}

fn validate_file_name(value: &str, field: &str) -> Result<(), PackError> {
    validate_relative_path(value, field)?;
    if Path::new(value).components().count() != 1 {
        return Err(PackError::new(format!(
            "{field} must be one safe path component"
        )));
    }
    Ok(())
}

fn validate_relative_path(value: &str, field: &str) -> Result<(), PackError> {
    if value.is_empty() || value.contains('\\') || value.contains('\0') {
        return Err(PackError::new(format!(
            "{field} is not a safe relative path"
        )));
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || path
            .components()
            .next()
            .and_then(|component| component.as_os_str().to_str())
            == Some(".git")
    {
        return Err(PackError::new(format!(
            "{field} is not a safe relative path: {value}"
        )));
    }
    Ok(())
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf, PackError> {
    let canonical = fs::canonicalize(path)
        .map_err(|error| PackError::with_context(&format!("could not resolve {label}"), error))?;
    if !canonical.is_dir() {
        return Err(PackError::new(format!("{label} is not a directory")));
    }
    Ok(canonical)
}

fn canonical_file(path: &Path, label: &str) -> Result<PathBuf, PackError> {
    let canonical = fs::canonicalize(path)
        .map_err(|error| PackError::with_context(&format!("could not resolve {label}"), error))?;
    if !canonical.is_file() {
        return Err(PackError::new(format!("{label} is not a file")));
    }
    Ok(canonical)
}

fn require_within(path: &Path, root: &Path, label: &str) -> Result<(), PackError> {
    if !path.starts_with(root) {
        return Err(PackError::new(format!(
            "{label} resolves outside the project root"
        )));
    }
    Ok(())
}

fn run_git<I, S>(project_root: &Path, arguments: I) -> Result<Output, PackError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new("git")
        .args(arguments)
        .current_dir(project_root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .map_err(|error| PackError::with_context("could not run git", error))
}

fn require_success(success: bool, context: &str, stderr: &[u8]) -> Result<(), PackError> {
    if success {
        return Ok(());
    }
    let detail = String::from_utf8_lossy(stderr);
    Err(PackError::new(format!("{context}: {}", detail.trim())))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
