#![forbid(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use book_render::{RenderError, RenderRequest, render};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("book-render: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), RenderError> {
    let Some(request) = parse_arguments(env::args().skip(1))? else {
        print_help();
        return Ok(());
    };
    let report = render(&request)?;
    println!(
        "rendered chapters={} parts={} assets={} output={}",
        report.chapter_count(),
        report.part_count(),
        report.asset_count(),
        report.output_path().display()
    );
    Ok(())
}

fn parse_arguments(
    arguments: impl IntoIterator<Item = String>,
) -> Result<Option<RenderRequest>, RenderError> {
    let mut book = None;
    let mut template = None;
    let mut output_dir = None;
    let mut iterator = arguments.into_iter();

    while let Some(argument) = iterator.next() {
        match argument.as_str() {
            "-h" | "--help" => return Ok(None),
            "--book" => set_once(&mut book, "--book", next_value(&mut iterator, "--book")?)?,
            "--template" => set_once(
                &mut template,
                "--template",
                next_value(&mut iterator, "--template")?,
            )?,
            "--output-dir" => set_once(
                &mut output_dir,
                "--output-dir",
                next_value(&mut iterator, "--output-dir")?,
            )?,
            other => {
                return Err(RenderError::new(format!(
                    "unknown argument {other:?}; use --help"
                )));
            }
        }
    }

    let book = book.ok_or_else(|| RenderError::new("missing required --book"))?;
    let template = template.ok_or_else(|| RenderError::new("missing required --template"))?;
    let output_dir = output_dir.ok_or_else(|| RenderError::new("missing required --output-dir"))?;

    Ok(Some(RenderRequest::new(book, template, output_dir)))
}

fn next_value(
    iterator: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<PathBuf, RenderError> {
    let value = iterator
        .next()
        .ok_or_else(|| RenderError::new(format!("{flag} requires a path")))?;
    if value.starts_with('-') {
        return Err(RenderError::new(format!(
            "{flag} requires a path, found {value:?}"
        )));
    }
    Ok(PathBuf::from(value))
}

fn set_once(target: &mut Option<PathBuf>, flag: &str, value: PathBuf) -> Result<(), RenderError> {
    if target.replace(value).is_some() {
        return Err(RenderError::new(format!(
            "{flag} may only be supplied once"
        )));
    }
    Ok(())
}

fn print_help() {
    println!(
        "book-render\n\n\
         Render canonical mdBook Markdown into a generated Typst project.\n\n\
         Usage:\n  \
         book-render --book <book-dir> --template <template.typ> --output-dir <dir>"
    );
}
