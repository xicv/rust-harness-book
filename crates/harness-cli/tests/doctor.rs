use std::error::Error;
use std::process::Command;

// ANCHOR: ch01_doctor_test
#[test]
fn doctor_reports_pinned_and_active_toolchain() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_harness-cli"))
        .arg("--doctor")
        .output()?;

    assert!(
        output.status.success(),
        "doctor command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout)?,
        concat!(
            "toolchain/status ok\n",
            "rust/pinned 1.98.0\n",
            "rustc/active 1.98.0\n",
            "cargo/active 1.98.0\n",
            "edition 2024\n",
            "resolver 3\n",
            "lockfile present\n",
        )
    );

    Ok(())
}
// ANCHOR_END: ch01_doctor_test
