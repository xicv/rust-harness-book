use book_render::render_markdown;

#[test]
fn markdown_contract_preserves_print_structure() {
    let source = concat!(
        "<div class=\"learning-card\">\n",
        "<p class=\"card-label\">Outcome / 本章成果</p>\n\n",
        "A **strong** term <span class=\"term-en\">Agent Harness</span>.\n\n",
        "- first\n",
        "- second\n\n",
        "```rust\n",
        "fn main() {}\n",
        "```\n",
        "</div>\n",
    );

    let Ok(rendered) = render_markdown(source) else {
        panic!("the Markdown fixture should render");
    };

    assert!(rendered.contains("#card("));
    assert!(rendered.contains("#card-label("));
    assert!(rendered.contains("#strong("));
    assert!(rendered.contains("#term("));
    assert!(rendered.contains("#bullet-list("));
    assert!(rendered.contains("#code-block("));
}
