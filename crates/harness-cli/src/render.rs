use harness_core::{Event, Item};

// ANCHOR: ch00_render_event
pub(crate) fn render_event(event: &Event) -> String {
    match event {
        Event::ThreadStarted { thread_id } => {
            format!("thread/started thread={thread_id}")
        }
        Event::TurnStarted { thread_id, turn_id } => {
            format!("turn/started thread={thread_id} turn={turn_id}")
        }
        Event::ItemCompleted {
            thread_id,
            turn_id,
            item: Item::User { id, text },
        } => {
            format!(
                "item/completed thread={thread_id} turn={turn_id} item={id} role=user text={text:?}"
            )
        }
        Event::ItemCompleted {
            thread_id,
            turn_id,
            item: Item::Assistant { id, text },
        } => {
            format!(
                "item/completed thread={thread_id} turn={turn_id} item={id} role=assistant text={text:?}"
            )
        }
        Event::TurnCompleted { thread_id, turn_id } => {
            format!("turn/completed thread={thread_id} turn={turn_id}")
        }
    }
}
// ANCHOR_END: ch00_render_event
