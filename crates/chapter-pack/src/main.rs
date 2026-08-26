#![forbid(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use chapter_pack::{BuildRequest, PackError, build_and_verify};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("chapter-pack: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), PackError> {
    let Some((manifest, output_directory)) = parse_arguments(env::args().skip(1))? else {
        print_help();
        return Ok(());
    };
    let project_root = env::current_dir()
        .map_err(|error| PackError::new(format!("could not resolve current directory: {error}")))?;
    let request = BuildRequest::new(project_root, manifest, output_directory);
    let report = build_and_verify(&request)?;
    println!(
        "packed source={} commands={} archive={} checksum={}",
        report.source_commit(),
        report.verified_command_count(),
        report.archive_path().display(),
        report.checksum_path().display()
    );
    Ok(())
}

fn parse_arguments(
    arguments: impl IntoIterator<Item = String>,
) -> Result<Option<(PathBuf, PathBuf)>, PackError> {
    let mut manifest = None;
    let mut output_directory = None;
    let mut iterator = arguments.into_iter();
    while let Some(argument) = iterator.next() {
        match argument.as_str() {
            "-h" | "--help" => return Ok(None),
            "--manifest" => set_once(
                &mut manifest,
                "--manifest",
                next_path(&mut iterator, "--manifest")?,
            )?,
            "--output-dir" => set_once(
                &mut output_directory,
                "--output-dir",
                next_path(&mut iterator, "--output-dir")?,
            )?,
            other => {
                return Err(PackError::new(format!(
                    "unknown argument {other:?}; use --help"
                )));
            }
        }
    }
    let manifest = manifest.ok_or_else(|| PackError::new("missing required --manifest"))?;
    let output_directory =
        output_directory.ok_or_else(|| PackError::new("missing required --output-dir"))?;
    Ok(Some((manifest, output_directory)))
}

fn next_path(
    iterator: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<PathBuf, PackError> {
    let value = iterator
        .next()
        .ok_or_else(|| PackError::new(format!("{flag} requires a path")))?;
    if value.starts_with('-') {
        return Err(PackError::new(format!(
            "{flag} requires a path, found {value:?}"
        )));
    }
    Ok(PathBuf::from(value))
}

fn set_once(target: &mut Option<PathBuf>, flag: &str, value: PathBuf) -> Result<(), PackError> {
    if target.replace(value).is_some() {
        return Err(PackError::new(format!("{flag} may only be supplied once")));
    }
    Ok(())
}

fn print_help() {
    println!(
        "chapter-pack\n\n\
         Build and verify one historical chapter workspace archive.\n\n\
         Usage:\n  \
         chapter-pack --manifest <chapter.toml> --output-dir <dir>"
    );
}
