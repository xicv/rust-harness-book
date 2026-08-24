#![forbid(unsafe_code)]
#![doc = "Core domain for the educational agent harness."]

use std::error::Error;
use std::fmt;

const DEMO_THREAD_ID: ThreadId = ThreadId::new(1);
const DEMO_TURN_ID: TurnId = TurnId::new(1);
const DEMO_USER_ITEM_ID: ItemId = ItemId::new(1);
const DEMO_ASSISTANT_ITEM_ID: ItemId = ItemId::new(2);

// ANCHOR: ch00_domain_types
/// Identifies one conversation container in the teaching harness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThreadId(u64);

impl ThreadId {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for ThreadId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.0;
        write!(formatter, "thread-{value}")
    }
}

/// Identifies one user-to-assistant exchange inside a thread.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TurnId(u64);

impl TurnId {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for TurnId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.0;
        write!(formatter, "turn-{value}")
    }
}

/// Identifies one user or assistant item inside a turn.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemId(u64);

impl ItemId {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.0;
        write!(formatter, "item-{value}")
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
// ANCHOR_END: ch00_domain_types

/// The complete observable result of one synchronous teaching turn.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnOutcome {
    events: Vec<Event>,
}

impl TurnOutcome {
    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }
}

/// Validation failures that stop a turn before model work begins.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunTurnError {
    BlankPrompt,
}

impl fmt::Display for RunTurnError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankPrompt => {
                formatter.write_str("prompt must contain a non-whitespace character")
            }
        }
    }
}

impl Error for RunTurnError {}

/// A concrete, deterministic model used before the book introduces provider traits.
pub struct EchoModel;

impl EchoModel {
    #[must_use]
    pub fn respond(prompt: &str) -> String {
        format!("Echo: {prompt}")
    }
}

// ANCHOR: ch00_run_turn
/// Runs one complete offline turn with the concrete echo model.
///
/// # Errors
///
/// Returns [`RunTurnError::BlankPrompt`] before model work starts when the
/// prompt contains no non-whitespace character.
pub fn run_turn(prompt: &str) -> Result<TurnOutcome, RunTurnError> {
    run_turn_with_model(prompt, EchoModel::respond)
}

/// Runs one complete turn through a small function seam.
///
/// This is intentionally simpler than a provider trait. A later chapter will
/// introduce a trait when the harness has more than one real provider shape.
///
/// # Errors
///
/// Returns [`RunTurnError::BlankPrompt`] before calling `model` when the prompt
/// contains no non-whitespace character.
pub fn run_turn_with_model(
    prompt: &str,
    model: fn(&str) -> String,
) -> Result<TurnOutcome, RunTurnError> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err(RunTurnError::BlankPrompt);
    }

    let assistant_text = model(prompt);
    let events = vec![
        Event::ThreadStarted {
            thread_id: DEMO_THREAD_ID,
        },
        Event::TurnStarted {
            thread_id: DEMO_THREAD_ID,
            turn_id: DEMO_TURN_ID,
        },
        Event::ItemCompleted {
            thread_id: DEMO_THREAD_ID,
            turn_id: DEMO_TURN_ID,
            item: Item::User {
                id: DEMO_USER_ITEM_ID,
                text: prompt.to_owned(),
            },
        },
        Event::ItemCompleted {
            thread_id: DEMO_THREAD_ID,
            turn_id: DEMO_TURN_ID,
            item: Item::Assistant {
                id: DEMO_ASSISTANT_ITEM_ID,
                text: assistant_text,
            },
        },
        Event::TurnCompleted {
            thread_id: DEMO_THREAD_ID,
            turn_id: DEMO_TURN_ID,
        },
    ];

    Ok(TurnOutcome { events })
}
// ANCHOR_END: ch00_run_turn
