//! A log line for each size the primary window settles at, extracted from Arkham's
//! tabletop health plugin.
//!
//! A desktop that scales a window can make the image the renderer fills far larger
//! than the display: on one 3840×2160 panel at GNOME's 125%, Xwayland gives a game
//! scale factor 2 and a window of up to 6144×3456 (#83). The size a table renders at bears on
//! its GPU load and on judging its layout at a player's display, so a game's log
//! should say it.
//!
//! [`WindowSizeLogPlugin`] logs, at `info`,
//! `Window: {W}×{H} physical pixels at scale factor {S}` for the window marked
//! [`PrimaryWindow`], once its physical size and scale factor have held unchanged
//! for half a second, and again each time they settle at a new value. A drag logs
//! where it ends, not every step; a return to the last logged size before a new one
//! settles logs nothing. The hold is timed by Bevy's [`Time<Real>`], which
//! `TimePlugin` keeps, as `DefaultPlugins` and `MinimalPlugins` include it.
//!
//! Other windows are never read, and with no primary window it logs nothing. The
//! agent seat's surface ([`capture::install`](crate::capture::install)) is a logical
//! primary window, so a table that adds the plugin at the seat logs the surface's
//! size there: add it to your windowed table only.
//!
//! What it leaves to you: whether a table adds it, and the log subscriber and filter
//! that decide where the line goes. Bevy's `LogPlugin` shows `info` by default.
//!
//! Tests hold the settle rule; and, for an `App` under `Time<Real>` set by hand, no
//! line before the primary window's size has held, one line once it has, another
//! when it settles at a new size, and none when the only window is not primary. No
//! test opens a real window or reads a desktop's scale.
use bevy::{prelude::*, window::PrimaryWindow};
use std::time::Duration;

/// How long a new size must hold before it is logged.
const SETTLE: Duration = Duration::from_millis(500);

/// Logs each size the primary window settles at: the module documentation has its
/// contract. Add it to a windowed table, in any schedule set-up that runs `Update`.
pub struct WindowSizeLogPlugin;

impl Plugin for WindowSizeLogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, log_window_size);
    }
}

/// The image the renderer fills, and the scale the desktop asks for.
#[derive(Clone, Copy, Debug, PartialEq)]
struct WindowSize {
    width: u32,
    height: u32,
    scale_factor: f32,
}

#[derive(Default)]
struct WindowSizeLog {
    logged: Option<WindowSize>,
    /// A size not yet logged, and the time it was first seen.
    pending: Option<(WindowSize, Duration)>,
}

impl WindowSizeLog {
    /// Returns a size to log once it differs from the last logged one and has held
    /// for [`SETTLE`].
    fn observe(&mut self, size: WindowSize, now: Duration) -> Option<WindowSize> {
        if self.logged == Some(size) {
            self.pending = None;
            return None;
        }
        match self.pending {
            Some((pending, since)) if pending == size => (now.saturating_sub(since) >= SETTLE)
                .then(|| {
                    self.logged = Some(size);
                    self.pending = None;
                    size
                }),
            _ => {
                self.pending = Some((size, now));
                None
            }
        }
    }
}

fn log_window_size(
    window: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time<Real>>,
    mut sizes: Local<WindowSizeLog>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let size = WindowSize {
        width: window.physical_width(),
        height: window.physical_height(),
        scale_factor: window.scale_factor(),
    };
    if let Some(size) = sizes.observe(size, time.elapsed()) {
        info!(
            "Window: {}×{} physical pixels at scale factor {}",
            size.width, size.height, size.scale_factor
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_size_is_logged_once_after_it_settles() {
        let size = |width| WindowSize {
            width,
            height: 2000,
            scale_factor: 2.0,
        };
        let at = Duration::from_millis;
        let mut log = WindowSizeLog::default();

        // The opening size holds for half a second before it is logged, once.
        assert_eq!(log.observe(size(3200), at(0)), None);
        assert_eq!(log.observe(size(3200), at(499)), None);
        assert_eq!(log.observe(size(3200), at(500)), Some(size(3200)));
        assert_eq!(log.observe(size(3200), at(5_000)), None);

        // A drag passes through sizes; only the one it ends on is logged.
        assert_eq!(log.observe(size(4000), at(6_000)), None);
        assert_eq!(log.observe(size(5000), at(6_400)), None);
        assert_eq!(log.observe(size(6010), at(6_800)), None);
        assert_eq!(log.observe(size(6010), at(7_299)), None);
        assert_eq!(log.observe(size(6010), at(7_300)), Some(size(6010)));

        // Returning to the logged size before a new one settles logs nothing.
        assert_eq!(log.observe(size(3200), at(8_000)), None);
        assert_eq!(log.observe(size(6010), at(8_100)), None);
        assert_eq!(log.observe(size(6010), at(9_000)), None);
    }
}
