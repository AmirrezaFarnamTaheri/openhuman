use super::*;

#[test]
fn wrong_target_error_identifies_the_required_agent() {
    let message = format_parse_error(&ParseError::WrongTarget {
        action: "react",
        expected: "trigger_reactor",
    });
    assert!(message.contains("react"));
    assert!(message.contains("target_agent `trigger_reactor`"));
}
