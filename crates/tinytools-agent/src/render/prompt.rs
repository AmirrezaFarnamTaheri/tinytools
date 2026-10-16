//! Compact, prompt-guided tool instructions for text-call dialects.
//!
//! The provider-neutral harness and host adapters use this block when native
//! tool schemas are unavailable. Keeping its compact schema rendering beside
//! the dialect prompt dispatch avoids a dependency from TinyTools back into a
//! particular harness.

use std::fmt::Write as _;

use serde_json::Value;
use tinytools::ToolSpec;

/// Maximum nesting depth expanded in compact object signatures.
const MAX_DEPTH: usize = 4;
/// Maximum object properties shown in a compact signature.
const MAX_PROPERTIES: usize = 16;
/// Maximum characters in a compact signature before clipping.
const MAX_SIGNATURE_CHARS: usize = 300;
/// Maximum characters in one argument description note.
const MAX_NOTE_CHARS: usize = 100;

/// Render a schema as the compact TypeScript-like type used in tool prompts.
#[must_use]
pub fn compact_type_signature(schema: &Value) -> String {
    let rendered = render(&sorted(schema), 0);
    if rendered.chars().count() <= MAX_SIGNATURE_CHARS {
        return rendered;
    }
    let mut clipped: String = rendered.chars().take(MAX_SIGNATURE_CHARS).collect();
    clipped.push('…');
    clipped
}

/// Render first-sentence notes for described top-level arguments.
#[must_use]
pub fn argument_notes(schema: &Value) -> Vec<String> {
    let schema = sorted(schema);
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return Vec::new();
    };
    properties
        .iter()
        .filter_map(|(name, property)| {
            let description = property.get("description")?.as_str()?.trim();
            (!description.is_empty())
                .then(|| format!("{name}: {}", first_sentence(description, MAX_NOTE_CHARS)))
        })
        .collect()
}

/// Render the compact JSON-in-tag prompt used by the harness's fallback path.
///
/// This intentionally keeps its historical wording and argument signatures
/// byte-for-byte stable while placing the implementation with the dialects.
#[must_use]
pub fn render_compact_tool_instructions(tools: &[ToolSpec]) -> String {
    let mut out = String::new();
    out.push_str("## Tool Use Protocol\n\n");
    out.push_str("To use a tool, wrap a JSON object in <tool_call></tool_call> tags:\n\n");
    out.push_str("<tool_call>\n");
    out.push_str(r#"{"name": "tool_name", "arguments": {"param": "value"}}"#);
    out.push('\n');
    out.push_str("</tool_call>\n\n");
    out.push_str("You may emit multiple tool calls in a single response. ");
    out.push_str("After execution, results appear in <tool_result> tags. ");
    out.push_str("Continue reasoning with the results until you can give a final answer.\n\n");
    out.push_str("### Available Tools\n\n");
    out.push_str("Arguments are shown as `{name: type, optional?: type}`.\n\n");
    for tool in tools {
        let _ = writeln!(out, "**{}**: {}", tool.name, tool.description);
        let _ = writeln!(out, "Arguments: `{}`", compact_type_signature(&tool.parameters));
        for note in argument_notes(&tool.parameters) {
            let _ = writeln!(out, "  - {note}");
        }
        out.push('\n');
    }
    out
}

fn first_sentence(text: &str, max_chars: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let end = collapsed
        .char_indices()
        .find(|&(index, ch)| {
            matches!(ch, '.' | '!' | '?')
                && collapsed[index + ch.len_utf8()..]
                    .chars()
                    .next()
                    .is_none_or(char::is_whitespace)
        })
        .map(|(index, _)| index);
    let sentence = end.map_or(collapsed.as_str(), |end| &collapsed[..end]);
    let mut clipped: String = sentence.chars().take(max_chars).collect();
    if clipped.len() < sentence.len() {
        clipped.push('…');
    }
    clipped
}

fn sorted(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries: Vec<_> = object.iter().collect();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            let mut result = serde_json::Map::new();
            for (key, value) in entries {
                result.insert(key.clone(), sorted(value));
            }
            Value::Object(result)
        }
        Value::Array(values) => Value::Array(values.iter().map(sorted).collect()),
        scalar => scalar.clone(),
    }
}

fn render(schema: &Value, depth: usize) -> String {
    let Some(object) = schema.as_object() else {
        return "unknown".to_string();
    };
    if let Some(values) = object.get("enum").and_then(Value::as_array) {
        return values
            .iter()
            .map(|value| match value {
                Value::String(value) => format!("{value:?}"),
                other => other.to_string(),
            })
            .collect::<Vec<_>>()
            .join(" | ");
    }
    if let Some(value) = object.get("const") {
        return match value {
            Value::String(value) => format!("{value:?}"),
            other => other.to_string(),
        };
    }
    for key in ["anyOf", "oneOf"] {
        if let Some(variants) = object.get(key).and_then(Value::as_array)
            && !variants.is_empty()
        {
            let mut rendered = Vec::new();
            for variant in variants {
                let variant = render(variant, depth);
                if !rendered.contains(&variant) {
                    rendered.push(variant);
                }
            }
            return rendered.join(" | ");
        }
    }
    if let Some(variants) = object.get("allOf").and_then(Value::as_array)
        && let Some(first) = variants.first()
    {
        return render(first, depth);
    }

    let kind = match object.get("type") {
        Some(Value::String(kind)) => kind.clone(),
        Some(Value::Array(kinds)) => {
            return kinds
                .iter()
                .filter_map(Value::as_str)
                .map(|kind| {
                    let mut single = object.clone();
                    single.insert("type".to_string(), Value::String(kind.to_string()));
                    render(&Value::Object(single), depth)
                })
                .collect::<Vec<_>>()
                .join(" | ");
        }
        _ if object.contains_key("properties") => "object".to_string(),
        _ if object.contains_key("items") => "array".to_string(),
        _ => return "unknown".to_string(),
    };
    match kind.as_str() {
        "object" => render_object(object, depth),
        "array" => {
            let items = object.get("items").map_or_else(
                || "unknown".to_string(),
                |items| {
                    if depth >= MAX_DEPTH { "unknown".to_string() } else { render(items, depth + 1) }
                },
            );
            if items.contains(' ') || items.contains('|') {
                format!("Array<{items}>")
            } else {
                format!("{items}[]")
            }
        }
        other => other.to_string(),
    }
}

fn render_object(object: &serde_json::Map<String, Value>, depth: usize) -> String {
    let Some(properties) = object.get("properties").and_then(Value::as_object) else {
        return "object".to_string();
    };
    if properties.is_empty() { return "{}".to_string(); }
    if depth >= MAX_DEPTH { return "object".to_string(); }
    let required: Vec<&str> = object
        .get("required")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let mut out = String::from("{");
    for (index, (name, property)) in properties.iter().enumerate() {
        if index == MAX_PROPERTIES {
            out.push_str(", …");
            break;
        }
        if index > 0 { out.push_str(", "); }
        let optional = if required.contains(&name.as_str()) { "" } else { "?" };
        let _ = write!(out, "{}{optional}: {}", render_property_name(name), render(property, depth + 1));
    }
    out.push('}');
    out
}

fn render_property_name(name: &str) -> String {
    let is_identifier = name
        .chars()
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_' || first == '$')
        && name.chars().all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '$');
    if is_identifier {
        name.to_string()
    } else {
        serde_json::to_string(name).unwrap_or_else(|_| format!("{name:?}"))
    }
}

#[cfg(test)]
#[path = "prompt_tests.rs"]
mod tests;
