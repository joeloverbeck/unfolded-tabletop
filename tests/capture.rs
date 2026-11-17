//! What Arkham's screen capture did at `6179043`, before it moved here: the
//! pixels it wrote, its windowless surface and its capture of that surface. One
//! thing differs on purpose and is marked below. No test here needs a graphics
//! adapter; each consumer's own checks cover a real render.
#![cfg(feature = "render")]

use bevy::{
    asset::RenderAssetUsages,
    camera::RenderTarget,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    window::{PrimaryWindow, WindowPlugin},
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use unfolded_tabletop::capture::{self, Ended, Missed, Pipelines, Surface, Unsaved};

/// A directory of this test's own, removed when the test ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(test: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "unfolded-tabletop-capture-{test}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn picture(format: TextureFormat, pixel: &[u8]) -> Image {
    Image::new_fill(
        Extent3d {
            width: 2,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixel,
        format,
        RenderAssetUsages::MAIN_WORLD,
    )
}

fn red() -> Image {
    picture(TextureFormat::Rgba8UnormSrgb, &[255, 0, 0, 255])
}

fn saved(path: &Path) -> image::DynamicImage {
    image::open(path).unwrap()
}

#[test]
fn an_eight_bit_picture_keeps_its_colour_and_loses_its_alpha() {
    let scratch = Scratch::new("eight-bit");
    let path = scratch.join("picture.png");
    // Alpha can carry brightness, not coverage: a zero must not blank the pixel.
    let image = picture(TextureFormat::Rgba8UnormSrgb, &[10, 20, 30, 0]);
    capture::write_image(image, &path).unwrap();
    let image::DynamicImage::ImageRgb8(pixels) = saved(&path) else {
        panic!("the file is not 8-bit RGB");
    };
    assert_eq!((pixels.width(), pixels.height()), (2, 1));
    assert!(pixels.pixels().all(|pixel| pixel.0 == [10, 20, 30]));
}

#[test]
fn a_floating_point_picture_is_converted_from_linear_light() {
    let scratch = Scratch::new("floating-point");
    let path = scratch.join("picture.png");
    // Linear 1.0, 0.2 and 0.0. The sRGB transfer function gives
    // 1.055 * 0.2^(1/2.4) - 0.055 = 0.4845, which is 123.6 of 255.
    let pixel: Vec<u8> = [1.0_f32, 0.2, 0.0, 7.0]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect();
    let image = picture(TextureFormat::Rgba32Float, &pixel);
    capture::write_image(image, &path).unwrap();
    let image::DynamicImage::ImageRgb8(pixels) = saved(&path) else {
        panic!("the file is not 8-bit RGB");
    };
    assert!(pixels.pixels().all(|pixel| pixel.0 == [255, 124, 0]));
}

#[test]
fn a_picture_whose_file_ending_names_no_format_is_refused_and_leaves_the_older_file() {
    let scratch = Scratch::new("refused");
    let unsupported = scratch.join("picture.unsupported");
    std::fs::write(&unsupported, "older file").unwrap();
    let refused = capture::write_image(red(), &unsupported).unwrap_err();
    assert!(matches!(refused, Unsaved::Unencoded(_)), "{refused:?}");
    assert_eq!(std::fs::read_to_string(&unsupported).unwrap(), "older file");

    let absent = scratch.join("absent/picture.png");
    let refused = capture::write_image(red(), &absent).unwrap_err();
    assert!(
        matches!(
            &refused,
            Unsaved::Unwritten {
                kind: std::io::ErrorKind::NotFound,
                ..
            }
        ),
        "{refused:?}"
    );
    // The words are the operating system's, as they were.
    assert_eq!(
        refused.to_string(),
        "No such file or directory (os error 2)"
    );
    assert!(!absent.parent().unwrap().exists());
}

/// A host with Bevy's window bookkeeping and no window event loop.
fn windowless(window: WindowPlugin) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, window));
    app.finish();
    app.cleanup();
    for _ in 0..3 {
        app.update();
    }
    app
}

#[test]
fn a_windowless_host_makes_no_window_and_does_not_exit_for_the_lack_of_one() {
    let mut app = windowless(capture::no_window());
    let mut windows = app.world_mut().query::<&Window>();
    assert_eq!(windows.iter(app.world()).count(), 0);
    assert!(app.should_exit().is_none());

    // Bevy's own settings without a window ask the host to exit at once.
    let app = windowless(WindowPlugin {
        primary_window: None,
        ..default()
    });
    assert!(app.should_exit().is_some());
}

/// A table's plugins as a game gives them: Bevy's window bookkeeping and its window
/// event loop, which opens a connection to the desktop when it is built.
struct Native;

impl PluginGroup for Native {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        bevy::app::PluginGroupBuilder::start::<Self>()
            .add_group(MinimalPlugins)
            .add(WindowPlugin::default())
            .add(bevy::winit::WinitPlugin::default())
    }
}

#[test]
fn the_windowless_configuration_leaves_out_the_window_event_loop() {
    let plugins = capture::windowless(Native);
    assert!(plugins.contains::<bevy::winit::WinitPlugin>());
    assert!(!plugins.enabled::<bevy::winit::WinitPlugin>());

    // Built with the event loop left out. Built, winit would refuse to start off the
    // main thread, as a test runs, and wherever no desktop exists, as in CI.
    let mut app = App::new();
    app.add_plugins(capture::windowless(Native));
    app.finish();
    app.cleanup();
    for _ in 0..3 {
        app.update();
    }
    let mut windows = app.world_mut().query::<&Window>();
    assert_eq!(windows.iter(app.world()).count(), 0);
    assert!(app.should_exit().is_none());
}

#[derive(Component)]
struct Seen;

/// A host with no renderer: what Bevy needs to hold an image and run startup.
fn host() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>();
    app
}

#[test]
fn the_surface_gives_the_named_camera_an_image_and_the_host_a_logical_window() {
    let mut app = host();
    capture::install::<Seen>(&mut app, UVec2::new(640, 360));
    assert_eq!(app.world().resource::<Surface>().camera(), None);
    let other = app.world_mut().spawn(Camera3d::default()).id();
    let seen = app.world_mut().spawn((Camera3d::default(), Seen)).id();
    app.update();

    let surface = app.world().resource::<Surface>();
    assert_eq!(surface.camera(), Some(seen));
    let image = surface.image();
    let target = app.world().get::<RenderTarget>(seen).unwrap();
    assert!(
        matches!(target, RenderTarget::Image(target) if target.handle == image),
        "{target:?}"
    );
    assert!(
        app.world()
            .get::<bevy::ui::IsDefaultUiCamera>(seen)
            .is_some()
    );
    assert!(
        !matches!(
            app.world().get::<RenderTarget>(other),
            Some(RenderTarget::Image(_))
        ),
        "only the named camera renders the surface"
    );

    let texture = app.world().resource::<Assets<Image>>().get(&image).unwrap();
    assert_eq!(texture.size(), UVec2::new(640, 360));
    assert_eq!(
        texture.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb
    );
    let mut windows = app
        .world_mut()
        .query_filtered::<&Window, With<PrimaryWindow>>();
    let window = windows.single(app.world()).unwrap();
    assert_eq!((window.width(), window.height()), (640.0, 360.0));
    assert!(window.focused);
}

fn renders_surface(app: &App, camera: Entity) -> bool {
    let image = app.world().resource::<Surface>().image();
    matches!(
        app.world().get::<RenderTarget>(camera),
        Some(RenderTarget::Image(target)) if target.handle == image
    )
}

#[test]
fn every_camera_marked_for_the_surface_renders_it_in_the_order_the_game_gave() {
    let mut app = host();
    // Marked before the surface exists, as a game whose scene is built first.
    let early = app
        .world_mut()
        .spawn((Camera3d::default(), capture::OnSurface))
        .id();
    capture::install::<Seen>(&mut app, UVec2::new(640, 360));
    let table = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            Camera {
                order: -1,
                ..default()
            },
            capture::OnSurface,
        ))
        .id();
    let ui = app
        .world_mut()
        .spawn((
            Camera2d,
            Camera {
                order: 1,
                ..default()
            },
            Seen,
        ))
        .id();
    let unmarked = app.world_mut().spawn(Camera3d::default()).id();
    app.update();
    // Marked after startup, as a view a game opens later.
    let later = app
        .world_mut()
        .spawn((Camera3d::default(), capture::OnSurface))
        .id();
    app.update();

    for camera in [early, table, ui, later] {
        assert!(
            renders_surface(&app, camera),
            "{camera} renders the surface"
        );
    }
    assert!(!renders_surface(&app, unmarked));
    let world = app.world();
    let default_ui = [early, table, ui, later, unmarked]
        .into_iter()
        .filter(|camera| world.get::<bevy::ui::IsDefaultUiCamera>(*camera).is_some())
        .collect::<Vec<_>>();
    assert_eq!(default_ui, [ui]);
    assert_eq!(world.resource::<Surface>().camera(), Some(ui));
    assert_eq!(world.get::<Camera>(table).unwrap().order, -1);
    assert_eq!(world.get::<Camera>(ui).unwrap().order, 1);
}

#[test]
fn a_host_without_the_named_camera_keeps_running_and_reports_no_camera() {
    let mut app = host();
    capture::install::<Seen>(&mut app, UVec2::new(640, 360));
    let other = app.world_mut().spawn(Camera3d::default()).id();
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(app.world().resource::<Surface>().camera(), None);
    assert!(!matches!(
        app.world().get::<RenderTarget>(other),
        Some(RenderTarget::Image(_))
    ));
}

#[test]
fn a_surface_on_a_host_with_no_render_world_finds_its_pipelines_ready() {
    let mut app = host();
    capture::install::<Seen>(&mut app, UVec2::new(640, 360));
    app.update();

    // Nothing is compiled where nothing renders. A stand-in renderer reports otherwise.
    let pipelines = app.world().resource::<Pipelines>().clone();
    assert!(pipelines.ready());
    pipelines.report(false);
    assert!(!Pipelines::of(&mut app).ready());
}

/// The captures this host has been asked for, with what each one captures.
fn requests(app: &mut App) -> Vec<(Entity, RenderTarget)> {
    let mut requests = app.world_mut().query::<(Entity, &Screenshot)>();
    requests
        .iter(app.world())
        .map(|(entity, screenshot)| (entity, screenshot.0.clone()))
        .collect()
}

/// One update of a host with no renderer: hand a picture back to every capture
/// asked for so far, as Bevy's renderer does a frame or more after the request.
fn renderer_hands_back(app: &mut App) -> bool {
    app.update();
    for (entity, _) in requests(app) {
        app.world_mut().trigger(ScreenshotCaptured {
            entity,
            image: red(),
        });
    }
    true
}

fn soon() -> Instant {
    Instant::now() + Duration::from_secs(30)
}

#[test]
fn a_capture_asks_for_the_given_image_and_returns_once_its_file_is_written() {
    let scratch = Scratch::new("given-image");
    let mut app = host();
    capture::install::<Seen>(&mut app, UVec2::new(2, 1));
    let image = app.world().resource::<Surface>().image();

    // Two ready steps, then the picture is asked for; it comes back on the third
    // step after that, not the first.
    let path = scratch.join("picture.png");
    let mut stepped = Vec::new();
    let mut asked_for = Vec::new();
    let result = capture::when_ready(
        &mut app,
        image.clone(),
        &path,
        soon(),
        |_| true,
        |app| {
            stepped.push(Instant::now());
            if stepped.len() < 5 {
                app.update();
                asked_for = requests(app);
                true
            } else {
                renderer_hands_back(app)
            }
        },
    );
    assert_eq!(result, Ok(()));
    assert_eq!(stepped.len(), 5);
    assert_eq!(saved(&path).to_rgb8().get_pixel(0, 0).0, [255, 0, 0]);
    assert_eq!(asked_for.len(), 1);
    assert!(
        matches!(&asked_for[0].1, RenderTarget::Image(target) if target.handle == image),
        "{:?}",
        asked_for[0].1
    );
    // The host is given at least two milliseconds between steps that wait for the
    // picture.
    for pair in stepped[2..].windows(2) {
        assert!(pair[1] - pair[0] >= Duration::from_millis(2), "{stepped:?}");
    }

    // The next capture's steps hand a picture back to the finished one as well,
    // which writes nothing more. Arkham's wrote its file again.
    std::fs::remove_file(&path).unwrap();
    let refused = scratch.join("absent/picture.png");
    let result = capture::when_ready(
        &mut app,
        image.clone(),
        &refused,
        soon(),
        |_| true,
        renderer_hands_back,
    );
    assert!(matches!(result, Err(Missed::Unsaved(_))), "{result:?}");
    assert!(!refused.exists());
    assert_eq!(requests(&mut app).len(), 2);
    assert!(!path.exists());

    // A picture that arrives on the last step the host permits, or on a step after
    // which the deadline has passed, is still written.
    for (name, host_stops) in [("stopped.png", true), ("late.png", false)] {
        let path = scratch.join(name);
        let deadline = Instant::now() + Duration::from_millis(200);
        let mut steps = 0;
        let result = capture::when_ready(
            &mut app,
            image.clone(),
            &path,
            if host_stops { soon() } else { deadline },
            |_| true,
            |app| {
                steps += 1;
                if steps < 3 {
                    app.update();
                    return true;
                }
                if !host_stops {
                    std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
                }
                renderer_hands_back(app);
                !host_stops
            },
        );
        assert_eq!((result, steps), (Ok(()), 3), "{name}");
        assert!(path.exists(), "{name}");
    }
}

#[test]
fn a_capture_when_ready_waits_for_two_consecutive_ready_frames_then_writes_its_file() {
    let scratch = Scratch::new("when-ready");
    let mut app = host();
    capture::install::<Seen>(&mut app, UVec2::new(2, 1));
    let image = app.world().resource::<Surface>().image();
    let path = scratch.join("picture.png");

    // The caller's condition holds on steps 2, 4, 5 and after; the stand-in renderer
    // reports its pipelines unready on step 5. Two consecutive frames first hold
    // after steps 6 and 7, so the picture is asked for on step 8.
    let pipelines = Pipelines::of(&mut app);
    let steps = std::cell::Cell::new(0);
    let mut asked_at = None;
    let result = capture::when_ready(
        &mut app,
        image,
        &path,
        soon(),
        |_| [2, 4, 5].contains(&steps.get()) || steps.get() >= 6,
        |app| {
            steps.set(steps.get() + 1);
            pipelines.report(steps.get() != 5);
            if asked_at.is_none() && !requests(app).is_empty() {
                asked_at = Some(steps.get());
            }
            renderer_hands_back(app)
        },
    );
    assert_eq!(result, Ok(()));
    assert_eq!(asked_at, Some(8));
    assert_eq!(steps.get(), 8);
    assert_eq!(saved(&path).to_rgb8().get_pixel(0, 0).0, [255, 0, 0]);
}

#[test]
fn a_capture_when_ready_says_whether_the_host_stopped_or_the_deadline_passed() {
    let scratch = Scratch::new("missed");
    let mut app = host();
    capture::install::<Seen>(&mut app, UVec2::new(2, 1));
    let image = app.world().resource::<Surface>().image();
    let shortly = || Instant::now() + Duration::from_millis(20);
    let stepping = |app: &mut App| {
        app.update();
        true
    };

    // Readiness never comes: the caller's condition never holds, or the stand-in
    // renderer's pipelines never compile. No picture is asked for.
    let never = scratch.join("never.png");
    let result = capture::when_ready(
        &mut app,
        image.clone(),
        &never,
        shortly(),
        |_| false,
        stepping,
    );
    assert_eq!(result, Err(Missed::Unready(Ended::Deadline)));
    let pipelines = Pipelines::of(&mut app);
    pipelines.report(false);
    let stepping_unready = |app: &mut App| {
        app.update();
        pipelines.report(false);
        true
    };
    let result = capture::when_ready(
        &mut app,
        image.clone(),
        &never,
        shortly(),
        |_| true,
        stepping_unready,
    );
    assert_eq!(result, Err(Missed::Unready(Ended::Deadline)));
    // A deadline already passed takes no step at all.
    let mut steps = 0;
    let result = capture::when_ready(
        &mut app,
        image.clone(),
        &never,
        Instant::now(),
        |_| true,
        |_| {
            steps += 1;
            true
        },
    );
    assert_eq!((result, steps), (Err(Missed::Unready(Ended::Deadline)), 0));
    pipelines.report(true);

    // The host exits while the table is still unready.
    let result = capture::when_ready(
        &mut app,
        image.clone(),
        &never,
        soon(),
        |_| false,
        |_| false,
    );
    assert_eq!(result, Err(Missed::Unready(Ended::Stopped)));
    assert!(requests(&mut app).is_empty());

    // The host exits after the picture is asked for: two ready steps, then one more.
    let exited = scratch.join("exited.png");
    let mut steps = 0;
    let result = capture::when_ready(
        &mut app,
        image.clone(),
        &exited,
        soon(),
        |_| true,
        |app| {
            steps += 1;
            app.update();
            steps < 3
        },
    );
    assert_eq!(result, Err(Missed::Incomplete(Ended::Stopped)));
    assert_eq!((steps, requests(&mut app).len()), (3, 1));

    // The deadline passes while the picture is awaited.
    let late = scratch.join("late.png");
    let result = capture::when_ready(
        &mut app,
        image.clone(),
        &late,
        shortly(),
        |_| true,
        stepping,
    );
    assert_eq!(result, Err(Missed::Incomplete(Ended::Deadline)));
    assert_eq!(requests(&mut app).len(), 2);

    // A picture that cannot be written is neither.
    let unwritable = scratch.join("absent/picture.png");
    let result = capture::when_ready(
        &mut app,
        image,
        &unwritable,
        soon(),
        |_| true,
        renderer_hands_back,
    );
    assert!(matches!(result, Err(Missed::Unsaved(_))), "{result:?}");
    assert!(!unwritable.exists());

    // That last capture's steps handed pictures back to both captures given up above.
    // Neither writes its file after its capture returned. Not as Arkham had it: there,
    // a picture that arrived after the capture was given up was still written.
    assert_eq!(requests(&mut app).len(), 3);
    assert!(!exited.exists());
    assert!(!late.exists());
    assert!(!never.exists());
}
