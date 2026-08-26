use std::error::Error;
use std::process::Command;

// ANCHOR: ch00_cli_test
#[test]
fn cli_prints_ordered_events() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_harness-cli"))
        .arg("hello")
        .output()?;

    assert!(
        output.status.success(),
        "CLI failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout)?,
        concat!(
            "thread/started thread=thread-1\n",
            "turn/started thread=thread-1 turn=turn-1\n",
            "item/completed thread=thread-1 turn=turn-1 item=item-1 role=user text=\"hello\"\n",
            "item/completed thread=thread-1 turn=turn-1 item=item-2 role=assistant text=\"Echo: hello\"\n",
            "turn/completed thread=thread-1 turn=turn-1\n",
        )
    );

    Ok(())
}
// ANCHOR_END: ch00_cli_test
