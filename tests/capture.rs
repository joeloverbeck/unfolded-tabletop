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
use unfolded_tabletop::capture::{self, Failure, Surface};

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
    assert!(capture::write_image(red(), &unsupported).is_err());
    assert_eq!(std::fs::read_to_string(&unsupported).unwrap(), "older file");

    let absent = scratch.join("absent/picture.png");
    assert!(capture::write_image(red(), &absent).is_err());
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
    let scratch = Scratch::new("image-to-file");
    let mut app = host();
    capture::install::<Seen>(&mut app, UVec2::new(2, 1));
    let image = app.world().resource::<Surface>().image();

    let path = scratch.join("picture.png");
    let mut stepped = Vec::new();
    let mut asked_for = Vec::new();
    let result = capture::image_to_file(&mut app, image.clone(), &path, soon(), |app| {
        stepped.push(Instant::now());
        // The picture comes back on the third update, not the first.
        if stepped.len() < 3 {
            app.update();
            asked_for = requests(app);
            true
        } else {
            renderer_hands_back(app)
        }
    });
    assert_eq!(result, Ok(()));
    assert_eq!(stepped.len(), 3);
    assert_eq!(saved(&path).to_rgb8().get_pixel(0, 0).0, [255, 0, 0]);
    assert_eq!(asked_for.len(), 1);
    assert!(
        matches!(&asked_for[0].1, RenderTarget::Image(target) if target.handle == image),
        "{:?}",
        asked_for[0].1
    );
    // The host is given at least two milliseconds between steps.
    for pair in stepped.windows(2) {
        assert!(pair[1] - pair[0] >= Duration::from_millis(2), "{stepped:?}");
    }

    // The next capture's step hands a picture back to the finished one as well,
    // which writes nothing more. Arkham's wrote its file again.
    std::fs::remove_file(&path).unwrap();
    let refused = scratch.join("absent/picture.png");
    let result = capture::image_to_file(&mut app, image, &refused, soon(), renderer_hands_back);
    assert!(matches!(result, Err(Failure::Unsaved(_))), "{result:?}");
    assert!(!refused.exists());
    assert_eq!(requests(&mut app).len(), 2);
    assert!(!path.exists());
}

#[test]
fn a_capture_that_does_not_come_back_is_incomplete() {
    let scratch = Scratch::new("incomplete");
    let mut app = host();
    capture::install::<Seen>(&mut app, UVec2::new(2, 1));
    let image = app.world().resource::<Surface>().image();

    // The host stops: one step is taken and no more.
    let stopped = scratch.join("stopped.png");
    let mut steps = 0;
    let result = capture::image_to_file(&mut app, image.clone(), &stopped, soon(), |_| {
        steps += 1;
        false
    });
    assert_eq!(result, Err(Failure::Incomplete));
    assert_eq!(steps, 1);

    // The deadline passes while the host keeps stepping.
    let late = scratch.join("late.png");
    let deadline = Instant::now() + Duration::from_millis(20);
    let result = capture::image_to_file(&mut app, image.clone(), &late, deadline, |app| {
        app.update();
        true
    });
    assert_eq!(result, Err(Failure::Incomplete));

    // A picture that arrives on the last permitted step is still written.
    let last = scratch.join("last.png");
    let result = capture::image_to_file(&mut app, image, &last, Instant::now(), |app| {
        renderer_hands_back(app);
        false
    });
    assert_eq!(result, Ok(()));
    assert!(last.exists());

    // Not as Arkham had it: there, a picture that arrived after the capture was
    // given up was still written, under a refusal that said no file was. That
    // last step handed pictures back to both captures given up above.
    assert_eq!(requests(&mut app).len(), 3);
    assert!(!stopped.exists());
    assert!(!late.exists());
}
