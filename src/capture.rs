//! Screen capture extracted from Arkham: a Bevy screenshot written to an image
//! file with timed diagnostics, and a surface without a window whose image can be
//! captured. Consumers decide when a capture is taken, what must be ready first,
//! how updates are stepped and what a failed capture means.
use bevy::{
    camera::RenderTarget,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    window::{ExitCondition, PrimaryWindow},
};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Write the picture when the renderer hands it back. `elapsed` is the consumer's
/// clock at the request; the diagnostics count on from it.
pub fn save_to_disk(path: PathBuf, elapsed: Duration) -> impl FnMut(On<ScreenshotCaptured>) {
    let mut write = writer(path, elapsed);
    move |captured| {
        let _ = write(&captured.image);
    }
}

/// Write the picture, then exit: successfully only when the file was saved.
pub fn save_to_disk_and_exit(
    path: PathBuf,
    elapsed: Duration,
) -> impl FnMut(On<ScreenshotCaptured>, MessageWriter<AppExit>) {
    let mut write = writer(path, elapsed);
    move |captured, mut exit| {
        exit.write(if write(&captured.image).is_ok() {
            AppExit::Success
        } else {
            AppExit::error()
        });
    }
}

/// The timed write behind both observers, for a consumer whose exit has further
/// conditions. It logs the request now and the actual outcome of each write.
pub fn writer(path: PathBuf, elapsed: Duration) -> impl FnMut(&Image) -> Result<(), String> {
    let requested = Instant::now();
    info!(elapsed_seconds = elapsed.as_secs_f64(), path = %path.display(), "Screenshot requested");
    move |image| {
        let span = info_span!("Screenshot write", write_started_seconds = (elapsed + requested.elapsed()).as_secs_f64(), path = %path.display());
        let _entered = span.enter();
        let result = write_image(image.clone(), &path);
        match &result {
            Ok(()) => info!("Screenshot saved to {}", path.display()),
            Err(error) => error!("Cannot save screenshot: {error}"),
        }
        info!(
            elapsed_seconds = (elapsed + requested.elapsed()).as_secs_f64(),
            "Screenshot writer returned"
        );
        result
    }
}

/// Write a captured picture as 8-bit RGB, in the format the path's extension names.
pub fn write_image(image: Image, path: &Path) -> Result<(), String> {
    let image = image
        .try_into_dynamic()
        .map_err(|error| error.to_string())?;
    // Screenshots from a native window use an 8-bit surface. Preserve Bevy's
    // RGB output (alpha can carry HDR brightness), including linear-to-sRGB
    // conversion if a floating-point render target supplies the capture.
    let rgb = if matches!(
        image,
        image::DynamicImage::ImageRgb32F(_) | image::DynamicImage::ImageRgba32F(_)
    ) {
        let linear = image.into_rgb32f();
        image::RgbImage::from_fn(linear.width(), linear.height(), |x, y| {
            let [r, g, b] = linear.get_pixel(x, y).0;
            image::Rgb(Srgba::from(LinearRgba::rgb(r, g, b)).to_u8_array_no_alpha())
        })
    } else {
        image.to_rgb8()
    };
    rgb.save(path).map_err(|error| error.to_string())
}

/// Window settings for a host that never opens an operating-system window: no
/// primary window is made, and the application does not exit for the lack of
/// one. The host also leaves its window event loop out of its plugins, and steps
/// updates itself.
pub fn no_window() -> WindowPlugin {
    WindowPlugin {
        primary_window: None,
        exit_condition: ExitCondition::DontExit,
        ..default()
    }
}

/// The image a windowless host renders, and the camera that renders it.
#[derive(Resource)]
pub struct Surface {
    image: Handle<Image>,
    camera: Option<Entity>,
}

impl Surface {
    pub fn image(&self) -> Handle<Image> {
        self.image.clone()
    }

    /// `None` until startup has found the camera that [`install`] named.
    pub fn camera(&self) -> Option<Entity> {
        self.camera
    }
}

/// Give the host a logical primary window with nothing behind it and an image for
/// camera `C` to render. Bevy's picking ignores every pointer without a primary
/// window, and draws UI to the image only for the default UI camera.
pub fn install<C: Component>(app: &mut App, size: UVec2) {
    app.world_mut().spawn((
        Window {
            resolution: (size.x, size.y).into(),
            focused: true,
            ..default()
        },
        PrimaryWindow,
    ));
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            size.x,
            size.y,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    app.insert_resource(Surface {
        image,
        camera: None,
    })
    .add_systems(PostStartup, retarget::<C>);
}

fn retarget<C: Component>(
    mut commands: Commands,
    mut surface: ResMut<Surface>,
    camera: Single<Entity, With<C>>,
) {
    commands.entity(*camera).insert((
        RenderTarget::Image(surface.image.clone().into()),
        bevy::ui::IsDefaultUiCamera,
    ));
    surface.camera = Some(*camera);
}

/// Why a capture left no picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// The picture came back and could not be written.
    Unsaved(String),
    /// No picture came back: the host stopped stepping, or the deadline passed.
    Incomplete,
}

/// Capture `image` to `path`, returning once the file is written. `step` runs one
/// update of the caller's host, on the caller's clock, and says whether the host
/// can go on. The caller decides what must be ready before it asks.
pub fn image_to_file(
    app: &mut App,
    image: Handle<Image>,
    path: &Path,
    deadline: Instant,
    mut step: impl FnMut(&mut App) -> bool,
) -> Result<(), Failure> {
    let written: Arc<Mutex<Option<Result<(), String>>>> = Arc::default();
    let slot = written.clone();
    let target = path.to_owned();
    app.world_mut().spawn(Screenshot::image(image)).observe(
        move |captured: On<ScreenshotCaptured>| {
            *slot.lock().unwrap() = Some(write_image(captured.image.clone(), &target));
        },
    );
    loop {
        let proceed = step(app);
        if let Some(result) = written.lock().unwrap().take() {
            return result.map_err(Failure::Unsaved);
        }
        if !proceed || Instant::now() >= deadline {
            return Err(Failure::Incomplete);
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}
