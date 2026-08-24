use harness_core::{
    Event, Item, ItemId, RunTurnError, ThreadId, TurnId, run_turn, run_turn_with_model,
};

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

// ANCHOR: ch00_blank_prompt_test
#[test]
fn blank_prompt_is_rejected_before_turn_start() {
    fn model_must_not_run(_prompt: &str) -> String {
        panic!("model was called before prompt validation");
    }

    let result = run_turn_with_model(" \n\t ", model_must_not_run);

    assert_eq!(result, Err(RunTurnError::BlankPrompt));
}
// ANCHOR_END: ch00_blank_prompt_test

#[test]
fn completed_event_reuses_thread_and_turn_ids() {
    let outcome = match run_turn("hello") {
        Ok(outcome) => outcome,
        Err(error) => panic!("expected a completed turn, got {error}"),
    };

    let expected_thread_id = ThreadId::new(1);
    let expected_turn_id = TurnId::new(1);

    for event in outcome.events() {
        match event {
            Event::ThreadStarted { thread_id } => {
                assert_eq!(*thread_id, expected_thread_id);
            }
            Event::TurnStarted { thread_id, turn_id }
            | Event::ItemCompleted {
                thread_id, turn_id, ..
            }
            | Event::TurnCompleted { thread_id, turn_id } => {
                assert_eq!(*thread_id, expected_thread_id);
                assert_eq!(*turn_id, expected_turn_id);
            }
        }
    }
}
