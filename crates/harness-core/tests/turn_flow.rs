use harness_core::{Event, Item, ItemId, ThreadId, TurnId, run_turn};

// ANCHOR: ch00_ordered_events_test
#[test]
fn one_prompt_produces_ordered_events() {
    let outcome = match run_turn("hello") {
        Ok(outcome) => outcome,
        Err(error) => panic!("expected a completed turn, got {error}"),
    };

    assert_eq!(
        outcome.events(),
        &[
            Event::ThreadStarted {
                thread_id: ThreadId::new(1),
            },
            Event::TurnStarted {
                thread_id: ThreadId::new(1),
                turn_id: TurnId::new(1),
            },
            Event::ItemCompleted {
                thread_id: ThreadId::new(1),
                turn_id: TurnId::new(1),
                item: Item::User {
                    id: ItemId::new(1),
                    text: "hello".to_owned(),
                },
            },
            Event::ItemCompleted {
                thread_id: ThreadId::new(1),
                turn_id: TurnId::new(1),
                item: Item::Assistant {
                    id: ItemId::new(2),
                    text: "Echo: hello".to_owned(),
                },
            },
            Event::TurnCompleted {
                thread_id: ThreadId::new(1),
                turn_id: TurnId::new(1),
            },
        ]
    );
}
// ANCHOR_END: ch00_ordered_events_test
