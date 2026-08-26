// ANCHOR: first_inline_playground
#[derive(Debug)]
enum Event {
    ThreadStarted,
    TurnStarted,
    ItemCompleted {
        role: &'static str,
        text: String,
    },
    TurnCompleted,
}

fn run_turn(prompt: &str) -> Vec<Event> {
    let prompt = prompt.trim();

    vec![
        Event::ThreadStarted,
        Event::TurnStarted,
        Event::ItemCompleted {
            role: "user",
            text: prompt.to_owned(),
        },
        Event::ItemCompleted {
            role: "assistant",
            text: format!("Echo: {prompt}"),
        },
        Event::TurnCompleted,
    ]
}

fn main() {
    let events = run_turn("hello");

    assert_eq!(events.len(), 5);

    for event in events {
        println!("{event:?}");
    }
}
// ANCHOR_END: first_inline_playground
