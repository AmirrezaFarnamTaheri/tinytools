use super::*;

#[test]
fn explicit_dispatchers_and_native_fallback_preserve_their_formats() {
    for (choice, expected) in [
        ("native", ToolCallFormat::Native),
        ("xml", ToolCallFormat::Json),
        ("pformat", ToolCallFormat::PFormat),
        ("python", ToolCallFormat::Python),
        ("typescript", ToolCallFormat::TypeScript),
    ] {
        for supported in [false, true] {
            assert_eq!(ToolCallFormat::resolve(choice, supported), expected);
        }
    }
    for choice in ["auto", "unknown", ""] {
        assert_eq!(
            ToolCallFormat::resolve(choice, true),
            ToolCallFormat::Native
        );
        assert_eq!(ToolCallFormat::resolve(choice, false), ToolCallFormat::Json);
    }
}

#[test]
fn only_code_formats_select_a_code_style() {
    assert_eq!(
        ToolCallFormat::Python.code_style(),
        Some(crate::codecall::CodeStyle::Python)
    );
    assert_eq!(
        ToolCallFormat::TypeScript.code_style(),
        Some(crate::codecall::CodeStyle::TypeScript)
    );
    for format in [
        ToolCallFormat::PFormat,
        ToolCallFormat::Json,
        ToolCallFormat::Native,
    ] {
        assert_eq!(format.code_style(), None);
    }
}
