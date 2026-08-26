# Verify Chapter 1 / 验证第 1 章

Run these commands from the unpacked `rust-harness-ch01` directory:

```sh
cargo test --workspace --locked
cargo run -p harness-cli --locked -- --doctor
```

The second command must print exactly the content of `expected-output.txt` when
the pinned toolchain is active. The archive checksum is published beside the
ZIP file; verify it before unpacking when downloading from the book site.
