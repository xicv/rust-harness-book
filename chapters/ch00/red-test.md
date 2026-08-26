# First Red test

Create `crates/harness-core/tests/turn_flow.rs` with a public-behaviour test named:

```text
one_prompt_produces_ordered_events
```

## Red 1: missing public behaviour

The first run may report unresolved imports for the exact API named in the
chapter contract. This is intentional: the public behaviour does not exist yet.
It is not acceptable to fail because of a misspelled path, accidental syntax
error, or missing test setup.

## Red 2: failing assertion

Add only the smallest public type and function signatures needed for the test to
compile. Return an empty or otherwise incorrect result. Run the test again and
confirm that its behavioural assertion fails.

Only then add the smallest implementation needed to make the expected complete
event list pass.

## Observed evidence

- **Red 1** — commit `5424534ddadf03d077c64e12b1f3ca96d9b42ad2`, CI run
  `32786145608`: the intended lifecycle imports did not exist.
- **Rejected setup attempt** — commit
  `d5eb82656d3841da842b1688cdda636ea8a97f36`: the test still could not compile
  because the temporary error type lacked `Display`. This was a test-fixture/API
  setup defect, not the accepted behavioural Red.
- **Red 2** — commit `53de82af38256891fcc0fd74f306d27e904e48a2`, CI run
  `32786491118`: the test compiled and failed with `left: []` against the full
  five-event expectation.
- **Green** — commit `3fed46defa0189e4e1a8f5b7dc3ab61743209b08`, CI run
  `32786877627`: the full scaffold pipeline passed.
