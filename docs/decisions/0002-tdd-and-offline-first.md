# ADR 0002: TDD and offline-first verification

## Decision

Every chapter starts from observable behaviour and a meaningful failing test.
Normal verification is deterministic and offline.

## Why

- Learners see why the code exists.
- Refactoring remains safe.
- Model nondeterminism cannot hide harness bugs.
- The course is accessible without paid services.

## Cost

- Test-support code must be designed carefully.
- Live semantic quality needs a separate evaluation layer.
