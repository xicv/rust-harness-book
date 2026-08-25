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

#[test]
fn text_is_escaped_before_it_becomes_typst_source() {
    let Ok(rendered) = render_markdown("Model text: `a` and \\ \" quote.") else {
        panic!("the escaping fixture should render");
    };

    assert!(rendered.contains("#inline-code(\"a\")"));
    assert!(rendered.contains("\\\\"));
    assert!(rendered.contains("\\\""));
}

#[test]
fn unsupported_html_fails_closed() {
    let error = match render_markdown("<script>alert('no')</script>") {
        Ok(_) => panic!("unsupported HTML must not render"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("unsupported block HTML"));
}

#[test]
fn file_features_require_the_complete_renderer() {
    let error = match render_markdown("{{#include example.rs}}") {
        Ok(_) => panic!("fragment rendering must not access the filesystem"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("complete book renderer"));
}
