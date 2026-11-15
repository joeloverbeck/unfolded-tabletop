use std::time::Duration;
use unfolded_hover::Hover;

#[test]
fn first_opening_waits_for_the_consumers_dwell() {
    let mut hover = Hover::default();
    let dwell = Duration::from_millis(120);
    hover.update(Some("first"), true, Duration::from_secs(1), dwell);
    assert_eq!(hover.visible(), None);
    assert!(hover.waiting());
    hover.update(Some("first"), true, Duration::from_millis(119), dwell);
    assert_eq!(hover.visible(), None);
    hover.update(Some("first"), true, Duration::from_millis(1), dwell);
    assert_eq!(hover.visible(), Some(&"first"));
    assert!(!hover.waiting());
}

#[test]
fn switching_an_open_view_does_not_repeat_the_dwell() {
    let mut hover = Hover::default();
    let dwell = Duration::from_millis(80);
    hover.update(Some(1), true, Duration::ZERO, dwell);
    hover.update(Some(1), true, dwell, dwell);
    hover.update(Some(2), true, Duration::ZERO, dwell);
    assert_eq!(hover.visible(), Some(&2));
    assert!(!hover.waiting());
}

#[test]
fn quick_crossings_do_not_open_and_leaving_resets_the_wait() {
    let mut hover = Hover::default();
    let dwell = Duration::from_millis(100);
    hover.update(Some(1), true, Duration::ZERO, dwell);
    hover.update(Some(1), true, Duration::from_millis(90), dwell);
    hover.update(Some(2), true, Duration::from_millis(20), dwell);
    assert_eq!(hover.visible(), None);
    assert!(hover.waiting());
    hover.update(None, false, Duration::from_secs(1), dwell);
    assert_eq!(hover.visible(), None);
    assert!(!hover.waiting());
    hover.update(Some(1), true, Duration::from_secs(1), dwell);
    assert_eq!(hover.visible(), None);
}

#[test]
fn an_ineligible_target_dismisses_and_reentry_waits_again() {
    let mut hover = Hover::default();
    let dwell = Duration::from_millis(100);
    hover.update(Some(1), true, Duration::ZERO, dwell);
    hover.update(Some(1), true, dwell, dwell);
    hover.update(Some(2), false, Duration::ZERO, dwell);
    assert_eq!(hover.visible(), None);
    assert!(!hover.waiting());
    hover.update(Some(1), true, Duration::ZERO, dwell);
    assert_eq!(hover.visible(), None);
    assert!(hover.waiting());
}

#[test]
fn eligibility_changes_keep_time_already_rested_on_the_same_target() {
    let mut hover = Hover::default();
    let dwell = Duration::from_millis(100);
    hover.update(Some(1), false, Duration::ZERO, dwell);
    hover.update(Some(1), false, dwell, dwell);
    hover.update(Some(1), true, Duration::ZERO, dwell);
    assert_eq!(hover.visible(), Some(&1));
    hover.update(Some(1), false, Duration::ZERO, dwell);
    assert_eq!(hover.visible(), None);
}
