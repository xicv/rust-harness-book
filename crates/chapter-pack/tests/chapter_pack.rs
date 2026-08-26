use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use chapter_pack::{BuildRequest, build_and_verify};
use zip::ZipArchive;

static NEXT_TEMP_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

#[test]
fn chapter_zero_pack_is_reproducible_and_self_verifying() -> Result<(), Box<dyn Error>> {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = TemporaryDirectory::new()?;
    let request = BuildRequest::new(
        &project_root,
        project_root.join("chapter-packs/ch00.toml"),
        output.path(),
    );

    let first = build_and_verify(&request)?;
    let first_bytes = fs::read(first.archive_path())?;
    let second = build_and_verify(&request)?;

    assert_eq!(
        first.source_commit(),
        "3fed46defa0189e4e1a8f5b7dc3ab61743209b08"
    );
    assert_eq!(first.verified_command_count(), 2);
    assert_eq!(first_bytes, fs::read(second.archive_path())?);
    assert_eq!(fs::read_to_string(first.checksum_path())?.len(), 65);

    let archive_file = fs::File::open(first.archive_path())?;
    let mut archive = ZipArchive::new(archive_file)?;
    let names = (0..archive.len())
        .map(|index| archive.by_index(index).map(|file| file.name().to_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    assert!(names.contains(&"rust-harness-ch00/README.md".to_owned()));
    assert!(names.contains(&"rust-harness-ch00/LICENSE".to_owned()));
    assert!(names.contains(&"rust-harness-ch00/crates/harness-core/src/lib.rs".to_owned()));
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]));

    Ok(())
}

#[test]
fn pack_manifest_rejects_commands_that_escape_the_unpacked_workspace() -> Result<(), Box<dyn Error>>
{
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = TemporaryDirectory::new()?;
    let source_manifest = fs::read_to_string(project_root.join("chapter-packs/ch00.toml"))?;
    let unsafe_manifest = source_manifest.replace(
        "[\"cargo\", \"test\", \"--workspace\", \"--locked\"]",
        "[\"cargo\", \"test\", \"--manifest-path\", \"/tmp/outside/Cargo.toml\"]",
    );
    let manifest_path = project_root.join(format!(
        "target/chapter-pack-unsafe-command-test-{}.toml",
        std::process::id()
    ));
    fs::write(&manifest_path, unsafe_manifest)?;
    let _manifest_guard = TemporaryFile::new(manifest_path.clone());
    let request = BuildRequest::new(&project_root, manifest_path, output.path());

    let error = match build_and_verify(&request) {
        Ok(_) => panic!("unsafe Cargo arguments must be rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("supported offline form"));
    Ok(())
}

#[test]
fn chapter_one_pack_includes_and_verifies_the_toolchain_doctor() -> Result<(), Box<dyn Error>> {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = TemporaryDirectory::new()?;
    let request = BuildRequest::new(
        &project_root,
        project_root.join("chapter-packs/ch01.toml"),
        output.path(),
    );

    let report = build_and_verify(&request)?;

    assert_eq!(
        report.source_commit(),
        "decef67c89afba8e4eb095b0c16454e4aca97eb5"
    );
    assert_eq!(report.verified_command_count(), 2);
    let archive_file = fs::File::open(report.archive_path())?;
    let mut archive = ZipArchive::new(archive_file)?;
    assert!(
        archive
            .by_name("rust-harness-ch01/crates/harness-cli/src/doctor.rs")
            .is_ok()
    );

    Ok(())
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn new() -> Result<Self, std::io::Error> {
        let sequence = NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "rust-harness-chapter-pack-test-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct TemporaryFile {
    path: PathBuf,
}

impl TemporaryFile {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
