//! What Arkham's screen capture did at `6179043`, before it moved here: its
//! pixels, its exits and its windowless surface. No test here needs a graphics
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
    window::{ExitCondition, PrimaryWindow},
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
fn a_picture_that_cannot_be_written_leaves_the_older_file() {
    let scratch = Scratch::new("refused");
    let unsupported = scratch.join("picture.unsupported");
    std::fs::write(&unsupported, "older file").unwrap();
    assert!(capture::write_image(red(), &unsupported).is_err());
    assert_eq!(std::fs::read_to_string(&unsupported).unwrap(), "older file");
    assert!(capture::write_image(red(), &scratch.join("absent/picture.png")).is_err());
}

fn hand_back(world: &mut World, entity: Entity) {
    world.trigger(ScreenshotCaptured {
        entity,
        image: red(),
    });
}

#[test]
fn the_observers_write_the_file_and_exit_on_the_actual_result() {
    let scratch = Scratch::new("observers");
    let mut world = World::new();
    world.init_resource::<Messages<AppExit>>();
    for (name, succeeds) in [("saved.png", true), ("stale.unsupported", false)] {
        let path = scratch.join(name);
        std::fs::write(&path, "older file").unwrap();
        world.resource_mut::<Messages<AppExit>>().clear();
        let entity = world
            .spawn_empty()
            .observe(capture::save_to_disk_and_exit(path.clone(), Duration::ZERO))
            .id();
        hand_back(&mut world, entity);
        let exits: Vec<_> = world
            .resource::<Messages<AppExit>>()
            .iter_current_update_messages()
            .collect();
        assert_eq!(exits.len(), 1, "{name}");
        assert_eq!(exits[0].is_success(), succeeds, "{name}");
        assert_eq!(
            std::fs::read(&path)
                .unwrap()
                .starts_with(b"\x89PNG\r\n\x1a\n"),
            succeeds,
            "{name}"
        );
    }

    // Without an exit, the file is written and nothing else is asked of the host.
    world.resource_mut::<Messages<AppExit>>().clear();
    let path = scratch.join("kept-open.png");
    let entity = world
        .spawn_empty()
        .observe(capture::save_to_disk(path.clone(), Duration::from_secs(5)))
        .id();
    hand_back(&mut world, entity);
    assert!(
        std::fs::read(&path)
            .unwrap()
            .starts_with(b"\x89PNG\r\n\x1a\n")
    );
    assert!(world.resource::<Messages<AppExit>>().is_empty());

    // A consumer with further exit conditions uses the same timed write.
    let mut write = capture::writer(scratch.join("own-policy.png"), Duration::ZERO);
    assert_eq!(write(&red()), Ok(()));
    let mut write = capture::writer(scratch.join("absent/own-policy.png"), Duration::ZERO);
    assert!(write(&red()).is_err());
}

#[derive(Component)]
struct Seen;

fn host() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>();
    app
}

#[test]
fn a_windowless_host_makes_no_window_and_does_not_exit_for_the_lack_of_one() {
    let plugin = capture::no_window();
    assert!(plugin.primary_window.is_none());
    assert!(matches!(plugin.exit_condition, ExitCondition::DontExit));
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

    let size = app
        .world()
        .resource::<Assets<Image>>()
        .get(&image)
        .unwrap()
        .size();
    assert_eq!(size, UVec2::new(640, 360));
    let mut windows = app
        .world_mut()
        .query_filtered::<&Window, With<PrimaryWindow>>();
    let window = windows.single(app.world()).unwrap();
    assert_eq!((window.width(), window.height()), (640.0, 360.0));
    assert!(window.focused);
}

/// One update of a host with no renderer: hand back each requested capture, as
/// Bevy's renderer does a frame or more after the request.
fn renderer_hands_back(app: &mut App) -> bool {
    app.update();
    let mut requests = app.world_mut().query_filtered::<Entity, With<Screenshot>>();
    let requests: Vec<Entity> = requests.iter(app.world()).collect();
    for entity in requests {
        hand_back(app.world_mut(), entity);
    }
    true
}

fn soon() -> Instant {
    Instant::now() + Duration::from_secs(30)
}

#[test]
fn a_capture_returns_once_its_file_is_written() {
    let scratch = Scratch::new("image-to-file");
    let mut app = host();
    capture::install::<Seen>(&mut app, UVec2::new(2, 1));
    let image = app.world().resource::<Surface>().image();

    let path = scratch.join("picture.png");
    let mut steps = 0;
    let result = capture::image_to_file(&mut app, image.clone(), &path, soon(), |app| {
        steps += 1;
        // The picture comes back on the third update, not the first.
        if steps < 3 {
            app.update();
            true
        } else {
            renderer_hands_back(app)
        }
    });
    assert_eq!(result, Ok(()));
    assert_eq!(steps, 3);
    assert_eq!(saved(&path).to_rgb8().get_pixel(0, 0).0, [255, 0, 0]);

    let refused = scratch.join("absent/picture.png");
    let result = capture::image_to_file(&mut app, image, &refused, soon(), renderer_hands_back);
    assert!(matches!(result, Err(Failure::Unsaved(_))), "{result:?}");
    assert!(!refused.exists());
}

#[test]
fn a_capture_that_never_comes_back_is_incomplete_and_writes_nothing() {
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
    assert!(!stopped.exists());

    // The deadline passes while the host keeps stepping.
    let late = scratch.join("late.png");
    let deadline = Instant::now() + Duration::from_millis(20);
    let result = capture::image_to_file(&mut app, image.clone(), &late, deadline, |app| {
        app.update();
        true
    });
    assert_eq!(result, Err(Failure::Incomplete));
    assert!(!late.exists());

    // A picture that arrives on the last permitted step is still written.
    let last = scratch.join("last.png");
    let result = capture::image_to_file(&mut app, image, &last, Instant::now(), |app| {
        renderer_hands_back(app);
        false
    });
    assert_eq!(result, Ok(()));
    assert!(last.exists());
}
