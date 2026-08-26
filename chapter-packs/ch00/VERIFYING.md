# Verify Chapter 0 / 验证第 0 章

Run these commands from the unpacked `rust-harness-ch00` directory:

```sh
cargo test --workspace --locked
cargo run -p harness-cli --locked -- hello
```

The second command must print exactly the content of `expected-output.txt`.
The archive checksum is published beside the ZIP file; verify that checksum
before unpacking when downloading from the book site.
