fn escape_typst_string(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            value if value.is_control() => {
                let _ = write!(output, "\\u{{{:x}}}", value as u32);
            }
            value => output.push(value),
        }
    }
    output
}

fn is_remote_target(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://") || value.starts_with("data:")
}

fn canonical_existing(path: &Path, description: &str) -> Result<PathBuf, RenderError> {
    fs::canonicalize(path)
        .map_err(|error| io_context(&format!("resolve {description}"), path, error))
}

fn ensure_inside(root: &Path, path: &Path, description: &str) -> Result<(), RenderError> {
    if path.starts_with(root) {
        return Ok(());
    }
    Err(RenderError::new(format!(
        "{description} is outside project root {}: {}",
        root.display(),
        path.display()
    )))
}

fn common_ancestor(left: &Path, right: &Path) -> Option<PathBuf> {
    left.ancestors()
        .find(|ancestor| right.starts_with(ancestor))
        .map(Path::to_owned)
}

fn safe_output_directory(
    path: &Path,
    project_root: &Path,
    protected_paths: &[(&Path, &str)],
) -> Result<PathBuf, RenderError> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|error| RenderError::new(format!("read current directory: {error}")))?
            .join(path)
    };
    if fs::symlink_metadata(&absolute).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(RenderError::new(format!(
            "renderer output directory must not be a symlink: {}",
            absolute.display()
        )));
    }

    let output = resolve_without_creating(&absolute, "renderer output directory")?;
    ensure_inside(project_root, &output, "renderer output directory")?;
    ensure_disjoint_output(&output, protected_paths)?;

    let parent = output
        .parent()
        .ok_or_else(|| RenderError::new("renderer output directory has no parent"))?;
    fs::create_dir_all(parent)
        .map_err(|error| io_context("create renderer output parent", parent, error))?;
    let canonical_parent = canonical_existing(parent, "renderer output parent")?;
    let file_name = output
        .file_name()
        .ok_or_else(|| RenderError::new("renderer output directory has no final component"))?;
    let output = canonical_parent.join(file_name);
    ensure_inside(project_root, &output, "renderer output directory")?;
    ensure_disjoint_output(&output, protected_paths)?;
    Ok(output)
}

fn resolve_without_creating(path: &Path, description: &str) -> Result<PathBuf, RenderError> {
    let mut existing = path;
    let mut missing = Vec::new();
    while !existing.exists() {
        let name = existing.file_name().ok_or_else(|| {
            RenderError::new(format!(
                "{description} has an unresolved non-normal component: {}",
                path.display()
            ))
        })?;
        missing.push(name.to_owned());
        existing = existing.parent().ok_or_else(|| {
            RenderError::new(format!(
                "{description} has no existing ancestor: {}",
                path.display()
            ))
        })?;
    }
    let mut resolved = canonical_existing(existing, description)?;
    for component in missing.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn ensure_disjoint_output(
    output: &Path,
    protected_paths: &[(&Path, &str)],
) -> Result<(), RenderError> {
    for (protected, description) in protected_paths {
        let protected = resolve_without_creating(protected, description)?;
        if output.starts_with(&protected) || protected.starts_with(output) {
            return Err(RenderError::new(format!(
                "renderer output overlaps protected input {description}: output={} protected={}",
                output.display(),
                protected.display()
            )));
        }
    }
    Ok(())
}

fn sibling_work_path(output: &Path, purpose: &str) -> PathBuf {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("book-render");
    parent.join(format!(".{name}.{purpose}.{}", std::process::id()))
}

fn replace_output_directory(
    staging: &Path,
    output: &Path,
    backup: &Path,
) -> Result<(), RenderError> {
    if output.exists() {
        fs::rename(output, backup)
            .map_err(|error| io_context("backup previous renderer output", output, error))?;
    }
    if let Err(error) = fs::rename(staging, output) {
        if backup.exists() {
            let _ = fs::rename(backup, output);
        }
        return Err(io_context("publish generated Typst project", output, error));
    }
    remove_if_exists(backup, "previous renderer output backup")?;
    Ok(())
}

fn remove_if_exists(path: &Path, description: &str) -> Result<(), RenderError> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| io_context(&format!("inspect {description}"), path, error))?;
    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path)
            .map_err(|error| io_context(&format!("remove {description}"), path, error))
    } else {
        fs::remove_dir_all(path)
            .map_err(|error| io_context(&format!("remove {description}"), path, error))
    }
}

fn read_utf8(path: &Path, description: &str) -> Result<String, RenderError> {
    fs::read_to_string(path)
        .map_err(|error| io_context(&format!("read {description}"), path, error))
}

fn io_context(action: &str, path: &Path, error: io::Error) -> RenderError {
    RenderError::new(format!("{action} {}: {error}", path.display()))
}

fn format_error(_: fmt::Error) -> RenderError {
    RenderError::new("failed to build generated Typst source")
}
