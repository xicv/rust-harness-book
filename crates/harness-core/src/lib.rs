#![forbid(unsafe_code)]
#![doc = "Core domain for the educational agent harness."]

use std::fmt;

/// Marks the repository state while Chapter 0 is under construction.
pub const SCAFFOLD_STATUS: &str = "chapter-0-red-2";

/// Identifies one conversation container in the teaching harness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThreadId(u64);

impl ThreadId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

/// Identifies one user-to-assistant exchange inside a thread.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TurnId(u64);

impl TurnId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

/// Identifies one persisted or streamed item inside a turn.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemId(u64);

impl ItemId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

/// One user or assistant contribution to a turn.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Item {
    User { id: ItemId, text: String },
    Assistant { id: ItemId, text: String },
}

/// Observable lifecycle output returned by the harness core.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    ThreadStarted {
        thread_id: ThreadId,
    },
    TurnStarted {
        thread_id: ThreadId,
        turn_id: TurnId,
    },
    ItemCompleted {
        thread_id: ThreadId,
        turn_id: TurnId,
        item: Item,
    },
    TurnCompleted {
        thread_id: ThreadId,
        turn_id: TurnId,
    },
}

/// The complete observable result of one synchronous teaching turn.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnOutcome {
    events: Vec<Event>,
}

impl TurnOutcome {
    pub fn events(&self) -> &[Event] {
        &self.events
    }
}

/// Validation failures that can stop a turn before model work begins.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunTurnError {
    BlankPrompt,
}

impl fmt::Display for RunTurnError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankPrompt => formatter.write_str("prompt must not be blank"),
        }
    }
}

/// Runs one turn. Chapter 0 intentionally starts with an incomplete Red state.
pub fn run_turn(_prompt: &str) -> Result<TurnOutcome, RunTurnError> {
    Ok(TurnOutcome { events: Vec::new() })
}
