use super::*;

#[test]
fn description_protocol_keeps_the_frozen_legacy_prompt_bytes() {
    assert_eq!(
        XmlDialect::description_protocol(),
        include_str!("xml_description_protocol.golden.txt"),
    );
}
