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
