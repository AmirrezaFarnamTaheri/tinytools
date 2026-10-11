use super::*;

#[test]
fn prose_is_not_a_tool_call() {
    assert!(!looks_like_unexecuted_tool_call(
        "All done, the file is saved."
    ));
    assert!(!contains_tool_call_payload("All done."));
    assert_eq!(strip_tool_calls_from_response("  plain  "), "  plain  ");
}

#[test]
fn xml_and_json_payloads_are_recognised() {
    assert!(contains_tool_call_payload("<tool_call>{}</tool_call>"));
    assert!(contains_tool_call_payload("{\n  \"tool_calls\": []\n}"));
    assert!(looks_like_unexecuted_tool_call("{\"tool_use\": 1}"));
    assert!(!contains_tool_call_payload("{\"tool_use\": 1}"));
}

#[test]
fn stripping_keeps_only_the_prose() {
    let text = "Before\n<tool_call>{\"name\":\"x\"}</tool_call>\n\n\n\nAfter\n{\"tool_calls\": []}";
    assert_eq!(strip_tool_calls_from_response(text), "Before\n\n\nAfter");
}

#[test]
fn an_unclosed_tag_drops_the_tail() {
    assert_eq!(
        strip_tool_calls_from_response("Keep <tool_call>{\"a\":1"),
        "Keep"
    );
}

#[test]
fn a_pure_payload_strips_to_empty() {
    assert_eq!(
        strip_tool_calls_from_response("<tool_call>{}</tool_call>"),
        ""
    );
}

#[test]
fn marker_lines_and_repeated_spans_leave_prose_in_order() {
    let input = concat!(
        "Before  \n",
        "  \"tool_calls\": []\n",
        "{\"tool_use\":{}}\n",
        "<tool_call>a</tool_call><tool_call>b</tool_call>\n",
        "After  "
    );
    assert_eq!(strip_tool_calls_from_response(input), "Before\n\nAfter");
    assert_eq!(strip_tool_calls_from_response("  prose\n\n"), "  prose\n\n");
}

#[test]
fn tool_use_prose_is_a_broad_hint_without_a_call_shaped_payload() {
    let text = "The protocol calls it \"tool_use\".";
    assert!(looks_like_unexecuted_tool_call(text));
    assert!(!contains_tool_call_payload(text));
    assert_eq!(strip_tool_calls_from_response(text), "");
}
