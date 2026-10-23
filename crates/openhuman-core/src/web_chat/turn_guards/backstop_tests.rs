use super::*;
use tinyagents_harness::turn_deadline::{parse_backstop, TurnDeadline};

#[test]
fn parse_backstop_defaults_and_disables() {
    assert_eq!(parse_backstop(None), Some(Duration::from_secs(900)));
    assert_eq!(
        parse_backstop(Some("garbage")),
        Some(Duration::from_secs(900))
    );
    assert_eq!(
        parse_backstop(Some(" 120 ")),
        Some(Duration::from_secs(120))
    );
    assert_eq!(parse_backstop(Some("0")), None);
}

#[test]
fn web_backstop_is_opt_in_and_keeps_the_override() {
    assert_eq!(parse_web_backstop(None), None);
    assert_eq!(parse_web_backstop(Some("garbage")), None);
    assert_eq!(
        parse_web_backstop(Some(" 120 ")),
        Some(Duration::from_secs(120))
    );
    assert_eq!(parse_web_backstop(Some("0")), None);
    let deadline = TurnDeadline::starting_now(Duration::from_secs(120));
    assert_eq!(deadline.wind_down_after(), Duration::from_secs(96));
    assert_eq!(deadline.hard_stop_after(), Duration::from_secs(108));
}
