use super::*;
use serde_json::json;
use tinytools::ToolSpec;

#[test]
fn compact_json_prompt_preserves_the_harness_wire_text() {
    let tools = [ToolSpec {
        name: "read_file".to_string(),
        description: "Read a file.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Relative path to the file. Must stay inside the workspace."},
                "limit": {"type": "integer", "description": "Maximum number of lines."}
            },
            "required": ["path"]
        }),
    }];
    assert_eq!(
        render_compact_tool_instructions(&tools),
        "## Tool Use Protocol\n\n\
To use a tool, wrap a JSON object in <tool_call></tool_call> tags:\n\n\
<tool_call>\n\
{\"name\": \"tool_name\", \"arguments\": {\"param\": \"value\"}}\n\
</tool_call>\n\n\
You may emit multiple tool calls in a single response. After execution, results appear in <tool_result> tags. Continue reasoning with the results until you can give a final answer.\n\n\
### Available Tools\n\n\
Arguments are shown as `{name: type, optional?: type}`.\n\n\
**read_file**: Read a file.\n\
Arguments: `{limit?: integer, path: string}`\n\
\u{20}\u{20}- limit: Maximum number of lines\n\
\u{20}\u{20}- path: Relative path to the file\n\n"
    );
}

#[test]
fn signatures_cover_schema_shapes_and_are_stably_sorted() {
    let schema = json!({
        "type": "object",
        "properties": {
            "file-path": {"type": "string"},
            "rows": {"type": "array", "items": {"type": "object", "properties": {"id": {"type": "integer"}}, "required": ["id"]}},
            "union": {"anyOf": [{"type": "string"}, {"type": "null"}, {"type": "string"}]},
            "mode": {"enum": ["fast", "full"]},
            "fixed": {"const": true},
            "choice": {"oneOf": [{"type": "integer"}, {"type": "number"}]},
            "all": {"allOf": [{"type": "boolean"}]},
            "typed": {"type": ["string", "null"]}
        },
        "required": ["rows", "union", "mode", "fixed", "choice", "all", "typed"]
    });
    let rendered = compact_type_signature(&schema);
    assert!(rendered.contains("\"file-path\"?: string"));
    assert!(rendered.contains("rows: Array<{id: integer}>"));
    assert!(rendered.contains("union: string | null"));
    assert!(rendered.contains("mode: \"fast\" | \"full\""));
    assert!(rendered.contains("fixed: true"));
    assert!(rendered.contains("choice: integer | number"));
    assert!(rendered.contains("all: boolean"));
    assert!(rendered.contains("typed: string | null"));
    assert_eq!(compact_type_signature(&json!({})), "unknown");
    assert_eq!(compact_type_signature(&json!({"type":"object"})), "object");
    assert_eq!(
        compact_type_signature(&json!({"type":"object","properties":{}})),
        "{}"
    );
}

#[test]
fn signatures_clip_wide_and_deep_schemas_and_notes_to_first_sentence() {
    fn nested(depth: usize) -> serde_json::Value {
        if depth == 0 {
            return json!({"type":"string"});
        }
        json!({"type":"object","properties":{"child":nested(depth-1)},"required":["child"]})
    }
    assert!(compact_type_signature(&nested(8)).matches('{').count() <= MAX_DEPTH);
    let mut properties = serde_json::Map::new();
    for index in 0..60 {
        properties.insert(
            format!("property_name_that_is_long_for_clipping_{index}"),
            json!({"type":"string"}),
        );
    }
    let wide = compact_type_signature(&json!({"type":"object","properties":properties}));
    assert!(wide.chars().count() <= MAX_SIGNATURE_CHARS + 1);
    assert!(wide.ends_with('…'));
    assert_eq!(
        argument_notes(&json!({"type":"object"})),
        Vec::<String>::new()
    );
    assert_eq!(
        first_sentence(
            "  Path   may be e.g. nested/file. Ignore this. ",
            MAX_NOTE_CHARS
        ),
        "Path may be e.g. nested/file"
    );
    assert!(first_sentence(&"x".repeat(120), MAX_NOTE_CHARS).ends_with('…'));
}
