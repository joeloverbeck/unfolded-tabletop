//! Opening lifecycle for pointer-driven reading views.
//!
//! Extracted from mudsong. Consumers own target picking and eligibility, dwell
//! duration, content, actions, geometry, timing, dimming and animation
//! readiness. This is not a general tooltip system. Reset the lifecycle with
//! [`Hover::default`] when the consumer's view resets.

use std::time::Duration;

/// Waits before opening the first eligible target, switches immediately while
/// a view is open, and dismisses on an absent or ineligible target.
#[derive(Debug)]
pub struct Hover<T> {
    pointed_at: Option<T>,
    rested: Duration,
    visible: Option<T>,
    waiting: bool,
}

impl<T> Default for Hover<T> {
    fn default() -> Self {
        Self {
            pointed_at: None,
            rested: Duration::ZERO,
            visible: None,
            waiting: false,
        }
    }
}

impl<T: Clone + PartialEq> Hover<T> {
    /// Advances using the consumer's clock and picked target. The frame that
    /// changes the target starts its dwell at zero, regardless of `elapsed`.
    /// Eligibility may change without changing the target's identity, and the
    /// target keeps the dwell it has accumulated.
    pub fn update(
        &mut self,
        target: Option<T>,
        eligible: bool,
        elapsed: Duration,
        dwell: Duration,
    ) {
        if target == self.pointed_at {
            self.rested += elapsed;
        } else {
            self.pointed_at = target;
            self.rested = Duration::ZERO;
        }
        let over = self.pointed_at.clone().filter(|_| eligible);
        let ready = over
            .clone()
            .filter(|_| self.visible.is_some() || self.rested >= dwell);
        self.waiting = over.is_some() && ready.is_none();
        self.visible = ready;
    }

    /// Identity whose reading view should be displayed, if any.
    pub fn visible(&self) -> Option<&T> {
        self.visible.as_ref()
    }

    /// Whether dwell is still pending. Consumers combine this with their own
    /// animation and rendering readiness before claiming that a view is settled.
    pub fn waiting(&self) -> bool {
        self.waiting
    }
}
