//! Screen capture extracted from Arkham: a captured picture written to an image
//! file, and a surface without a window whose image can be captured. Consumers
//! decide when a capture is taken, what must be ready first, how updates are
//! stepped, how long a capture may take and what a failed one means.
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
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

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

/// What a capture's observer may still do, and what it did.
enum Request {
    Waiting,
    Written(Result<(), String>),
    /// The caller has its answer, written or given up, so nothing that arrives
    /// later is written.
    Ended,
}

/// Capture `image` to `path`, returning once the file is written. `step` runs one
/// update of the caller's host, on the caller's clock, and says whether the host
/// can go on; at least two milliseconds pass between steps. The caller decides
/// what must be ready before it asks.
///
/// Once the capture has returned, a picture that arrives for it is not written.
/// So one that ends [`Failure::Incomplete`] leaves no file behind later. Its
/// request stays in the world until Bevy has handed that picture back, because
/// Bevy marks the request when it does.
pub fn image_to_file(
    app: &mut App,
    image: Handle<Image>,
    path: &Path,
    deadline: Instant,
    mut step: impl FnMut(&mut App) -> bool,
) -> Result<(), Failure> {
    let request = Arc::new(Mutex::new(Request::Waiting));
    let slot = request.clone();
    let target = path.to_owned();
    app.world_mut().spawn(Screenshot::image(image)).observe(
        move |captured: On<ScreenshotCaptured>| {
            if matches!(*slot.lock().unwrap(), Request::Waiting) {
                let written = write_image(captured.image.clone(), &target);
                *slot.lock().unwrap() = Request::Written(written);
            }
        },
    );
    loop {
        let proceed = step(app);
        {
            let mut request = request.lock().unwrap();
            let outcome = match &*request {
                Request::Written(result) => Some(result.clone().map_err(Failure::Unsaved)),
                _ if !proceed || Instant::now() >= deadline => Some(Err(Failure::Incomplete)),
                _ => None,
            };
            if let Some(outcome) = outcome {
                *request = Request::Ended;
                return outcome;
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}
