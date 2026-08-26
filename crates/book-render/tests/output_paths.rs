use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use book_render::{GitBookExportRequest, RenderRequest, export_gitbook, render};

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(1);

#[test]
fn gitbook_output_equal_to_book_is_rejected_without_mutation() {
    let project = TestProject::new("gitbook-book");
    let sentinel = project.path("book/sentinel.txt");
    let error = export_error(GitBookExportRequest::new(
        project.path("book"),
        project.path("book"),
    ));

    assert_protected(&project, &sentinel, &error);
}

#[test]
fn gitbook_output_inside_book_source_is_rejected_without_mutation() {
    let project = TestProject::new("gitbook-source");
    let sentinel = project.path("book/src/sentinel.txt");
    let error = export_error(GitBookExportRequest::new(
        project.path("book"),
        project.path("book/src/generated"),
    ));

    assert_protected(&project, &sentinel, &error);
}

#[test]
fn gitbook_output_equal_to_git_control_path_is_rejected_without_mutation() {
    let project = TestProject::new("gitbook-git");
    let sentinel = project.path(".git");
    let error = export_error(GitBookExportRequest::new(
        project.path("book"),
        project.path(".git"),
    ));

    assert_protected(&project, &sentinel, &error);
}

#[test]
fn gitbook_output_containing_typst_template_is_rejected_without_mutation() {
    let project = TestProject::new("gitbook-template");
    let sentinel = project.path("typst/template.typ");
    let error = export_error(GitBookExportRequest::new(
        project.path("book"),
        project.path("typst"),
    ));

    assert_protected(&project, &sentinel, &error);
}

#[test]
fn typst_output_equal_to_book_is_rejected_without_mutation() {
    let project = TestProject::new("typst-book");
    let sentinel = project.path("book/sentinel.txt");
    let error = render_error(project.render_request(project.path("book")));

    assert_protected(&project, &sentinel, &error);
}

#[test]
fn typst_output_inside_book_source_is_rejected_without_mutation() {
    let project = TestProject::new("typst-source");
    let sentinel = project.path("book/src/sentinel.txt");
    let error = render_error(project.render_request(project.path("book/src/generated")));

    assert_protected(&project, &sentinel, &error);
}

#[test]
fn typst_output_containing_template_is_rejected_without_mutation() {
    let project = TestProject::new("typst-template");
    let sentinel = project.path("typst/template.typ");
    let error = render_error(project.render_request(project.path("typst")));

    assert_protected(&project, &sentinel, &error);
}

#[cfg(unix)]
#[test]
fn symlinked_output_parent_resolving_inside_source_is_rejected_without_mutation() {
    use std::os::unix::fs::symlink;

    let project = TestProject::new("symlinked-source");
    let sentinel = project.path("book/src/sentinel.txt");
    symlink(project.path("book/src"), project.path("linked-source"))
        .unwrap_or_else(|error| panic!("failed to create output-parent symlink: {error}"));
    let error = export_error(GitBookExportRequest::new(
        project.path("book"),
        project.path("linked-source/generated"),
    ));

    assert_protected(&project, &sentinel, &error);
}

#[test]
fn non_normal_output_targeting_source_is_rejected_before_parent_creation() {
    let project = TestProject::new("non-normal-source");
    let sentinel = project.path("book/src/sentinel.txt");
    let unused_parent = project.path("unused-parent");
    let error = export_error(GitBookExportRequest::new(
        project.path("book"),
        unused_parent.join("../book/src/generated"),
    ));

    assert!(error.contains("unresolved non-normal component"), "{error}");
    assert_eq!(read_bytes(&sentinel), b"protected sentinel\n");
    assert_no_work_residue(&project.root);
    assert!(
        !unused_parent.exists(),
        "renderer created a parent before rejecting a non-normal output"
    );
}

fn export_error(request: GitBookExportRequest) -> String {
    match export_gitbook(&request) {
        Ok(_) => panic!("an output overlapping protected inputs must fail"),
        Err(error) => error.to_string(),
    }
}

fn render_error(request: RenderRequest) -> String {
    match render(&request) {
        Ok(_) => panic!("an output overlapping protected inputs must fail"),
        Err(error) => error.to_string(),
    }
}

fn assert_protected(project: &TestProject, sentinel: &Path, error: &str) {
    assert!(error.contains("overlaps protected input"), "{error}");
    assert_eq!(read_bytes(sentinel), b"protected sentinel\n");
    assert_no_work_residue(&project.root);
}

fn assert_no_work_residue(root: &Path) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to list {}: {error}", root.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read directory entry: {error}"));
        let name = entry.file_name();
        let name = name.to_string_lossy();
        assert!(
            !name.contains(".staging.")
                && !name.contains(".backup.")
                && !name.contains(".gitbook-staging.")
                && !name.contains(".gitbook-backup."),
            "stale renderer work path remained: {}",
            entry.path().display()
        );
    }
}

struct TestProject {
    root: PathBuf,
}

impl TestProject {
    fn new(name: &str) -> Self {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "rust-harness-output-{name}-{}-{sequence}",
            std::process::id()
        ));
        if root.exists() {
            let _ = fs::remove_dir_all(&root);
        }
        create_dir(&root);
        let root = fs::canonicalize(&root)
            .unwrap_or_else(|error| panic!("failed to resolve {}: {error}", root.display()));
        let project = Self { root };
        project.write(
            "book/book.toml",
            concat!(
                "[book]\n",
                "title = \"Fixture Book\"\n",
                "authors = [\"Test Author\"]\n",
                "description = \"A protected-output fixture.\"\n",
                "language = \"zh-CN\"\n",
                "src = \"src\"\n",
            ),
        );
        project.write(
            "book/src/SUMMARY.md",
            "# Summary\n\n- [Introduction](index.md)\n",
        );
        project.write("book/src/index.md", "# Introduction\n\nFixture.\n");
        project.write("book/src/sentinel.txt", "protected sentinel\n");
        project.write("book/sentinel.txt", "protected sentinel\n");
        project.write("typst/template.typ", "protected sentinel\n");
        project.write(".git", "protected sentinel\n");
        project
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            create_dir(parent);
        }
        fs::write(&path, content)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
    }

    fn render_request(&self, output: PathBuf) -> RenderRequest {
        RenderRequest::new(self.path("book"), self.path("typst/template.typ"), output)
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path)
        .unwrap_or_else(|error| panic!("failed to create {}: {error}", path.display()));
}

fn read_bytes(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}
