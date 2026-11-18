#![cfg(feature = "render")]
//! The window-size log through a real `App`: Bevy's own clock, set by hand each
//! frame, and the log read back from a tracing subscriber on the test's thread.
use bevy::{
    ecs::schedule::SingleThreadedExecutor,
    log::{tracing, tracing_subscriber},
    prelude::*,
    time::TimeUpdateStrategy,
    window::{PrimaryWindow, WindowResolution},
};
use std::{
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use unfolded_tabletop::window_size::WindowSizeLogPlugin;

/// Everything the subscriber writes.
#[derive(Clone, Default)]
struct Written(Arc<Mutex<Vec<u8>>>);

impl io::Write for Written {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Written {
    fn window_lines(&self) -> Vec<String> {
        String::from_utf8(self.0.lock().unwrap().clone())
            .unwrap()
            .lines()
            .filter(|line| line.contains("Window: "))
            .map(str::to_owned)
            .collect()
    }
}

/// Runs `frames` with every log line written to `written`. The subscriber is
/// the test thread's own, so `Update` runs on that thread.
fn logged(written: &Written, frames: impl FnOnce()) {
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_writer({
            let written = written.clone();
            move || written.clone()
        })
        .finish();
    tracing::subscriber::with_default(subscriber, frames);
}

/// An app with Bevy's clock and the log, whose frames run at a given time.
struct Table {
    app: App,
    start: Instant,
}

impl Table {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins((bevy::time::TimePlugin, WindowSizeLogPlugin));
        app.edit_schedule(Update, |schedule| {
            schedule.set_executor(SingleThreadedExecutor::new());
        });
        Self {
            app,
            start: Instant::now(),
        }
    }

    fn frame_at(&mut self, milliseconds: u64) {
        self.app.insert_resource(TimeUpdateStrategy::ManualInstant(
            self.start + Duration::from_millis(milliseconds),
        ));
        self.app.update();
    }
}

#[test]
fn the_primary_window_size_is_logged_once_it_has_held_for_half_a_second() {
    let mut table = Table::new();
    let window = table
        .app
        .world_mut()
        .spawn((
            Window {
                resolution: WindowResolution::new(2560, 1440).with_scale_factor_override(2.0),
                ..default()
            },
            PrimaryWindow,
        ))
        .id();
    let written = Written::default();
    let mut before = vec![];
    let mut settled = vec![];
    logged(&written, || {
        table.frame_at(0);
        table.frame_at(250);
        table.frame_at(499);
        before = written.window_lines();
        table.frame_at(500);
        table.frame_at(5_000);
        settled = written.window_lines();
        // The desktop makes the window larger, and the new size holds.
        table
            .app
            .world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set_physical_resolution(6144, 3456);
        table.frame_at(6_000);
        table.frame_at(6_499);
        table.frame_at(6_500);
        table.frame_at(9_000);
    });
    assert_eq!(before, Vec::<String>::new());
    assert_eq!(settled.len(), 1, "{settled:?}");
    assert!(
        settled[0].ends_with("Window: 2560×1440 physical pixels at scale factor 2"),
        "{settled:?}"
    );
    let lines = written.window_lines();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(
        lines[1].ends_with("Window: 6144×3456 physical pixels at scale factor 2"),
        "{lines:?}"
    );
}

#[test]
fn nothing_is_logged_without_a_primary_window() {
    let mut table = Table::new();
    // A window that is not the primary one, as a game's second window would be.
    table.app.world_mut().spawn(Window::default());
    let written = Written::default();
    logged(&written, || {
        for milliseconds in [0, 500, 1_000, 5_000] {
            table.frame_at(milliseconds);
        }
    });
    assert_eq!(written.window_lines(), Vec::<String>::new());
}
