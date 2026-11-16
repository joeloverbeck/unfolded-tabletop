//! Screen capture extracted from Arkham: a host's plugins without a window
//! ([`windowless`]), a surface whose image every marked camera renders ([`install`],
//! [`OnSurface`]), and one loop that waits until the table is ready and writes a
//! picture of that image ([`when_ready`]), for the agent seat and for a capture taken
//! with no seat. A picture is written as 8-bit RGB ([`write_image`]).
//!
//! Consumers decide when a capture is taken, what besides the render pipelines must be
//! ready first, how updates are stepped and on what clock, how long a capture may take,
//! what a failed one says, the order their cameras draw in, and whether rendering is
//! pipelined. Captures of a native window are theirs.
use bevy::{
    app::PluginGroupBuilder,
    camera::RenderTarget,
    prelude::*,
    render::{
        Render, RenderApp,
        render_resource::{CachedPipelineState, PipelineCache, TextureFormat},
        renderer::render_system,
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    window::{ExitCondition, PrimaryWindow},
    winit::WinitPlugin,
};
use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

/// Write a captured picture as 8-bit RGB, in the format the path's extension names.
/// Alpha is dropped, and a floating-point picture is converted from linear light.
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
/// one. [`windowless`] sets them and also leaves the window event loop out.
pub fn no_window() -> WindowPlugin {
    WindowPlugin {
        primary_window: None,
        exit_condition: ExitCondition::DontExit,
        ..default()
    }
}

/// A host's plugins with no operating-system window: [`no_window`]'s settings, and
/// no window event loop, so no connection to a desktop is made and nothing exits for
/// the lack of one. The host steps updates itself. `plugins` must hold Bevy's
/// `WindowPlugin`, as `DefaultPlugins` does; whether rendering is pipelined stays the
/// game's.
pub fn windowless(plugins: impl PluginGroup) -> PluginGroupBuilder {
    let plugins = plugins.build().set(no_window());
    if plugins.contains::<WinitPlugin>() {
        plugins.disable::<WinitPlugin>()
    } else {
        plugins
    }
}

/// The image a windowless host renders, and the camera named to render it.
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

/// Marks a camera that renders to the windowless surface's image, beside the camera
/// [`install`] names: a 3D table under a UI camera, or a second view drawn over the
/// first. Each camera keeps its own order, viewport and clear colour, so the game still
/// decides what draws over what. Mark it when, and wherever, the camera is spawned:
/// without a surface the mark does nothing, so a windowed start can carry it too.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct OnSurface;

/// Whether the render pipelines are ready, as the render world found them after its
/// last frame: none is queued and every one compiled. [`install`] adds the finding
/// while the host is built, before pipelined rendering can move the render world to a
/// thread of its own, and [`when_ready`] waits on it before a capture, for the agent
/// seat as for a capture with no seat, so that the capture omits nothing. A host with no render world compiles nothing: it is ready
/// unless something reports otherwise, as a renderer that is a stand-in does.
#[derive(Resource, Clone)]
pub struct Pipelines(Arc<AtomicBool>);

impl Pipelines {
    /// The host's finding, or a ready one for a host that has none.
    pub fn of(app: &mut App) -> Self {
        app.world_mut()
            .get_resource_or_insert_with(|| Self(Arc::new(AtomicBool::new(true))))
            .clone()
    }

    /// Whether the last finding was that every pipeline is ready.
    pub fn ready(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    /// Record what the render world found after a frame. Only the render world and a
    /// stand-in for it report.
    pub fn report(&self, ready: bool) {
        self.0.store(ready, Ordering::Release);
    }
}

/// The render world's finding after each frame it renders.
fn find_pipelines(pipelines: Res<Pipelines>, cache: Res<PipelineCache>) {
    pipelines.report(
        cache.waiting_pipelines().next().is_none()
            && cache
                .pipelines()
                .all(|pipeline| matches!(pipeline.state, CachedPipelineState::Ok(_))),
    );
}

/// Give the host a logical primary window with nothing behind it and an image for
/// camera `C`, and every camera marked [`OnSurface`], to render. Bevy's picking ignores
/// every pointer without a primary window, and draws UI to the image only for the
/// default UI camera, which `C` becomes. Where the host
/// has a render world, it also finds whether the render pipelines are ready
/// ([`Pipelines`]): call it after the render plugins are added.
pub fn install<C: Component>(app: &mut App, size: UVec2) {
    let render = app.get_sub_app_mut(RenderApp);
    let pipelines = Pipelines(Arc::new(AtomicBool::new(render.is_none())));
    if let Some(render) = render {
        render
            .insert_resource(pipelines.clone())
            .add_systems(Render, find_pipelines.after(render_system));
    }
    app.insert_resource(pipelines);
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
    let marked = app
        .world_mut()
        .query_filtered::<Entity, With<OnSurface>>()
        .iter(app.world())
        .collect::<Vec<_>>();
    for camera in marked {
        app.world_mut().entity_mut(camera).insert(renders(&image));
    }
    app.insert_resource(Surface {
        image,
        camera: None,
    })
    .add_observer(render_surface)
    .add_systems(PostStartup, name_camera::<C>);
}

/// A camera marked for the surface renders its image.
fn render_surface(marked: On<Add<OnSurface>>, surface: Res<Surface>, mut commands: Commands) {
    commands
        .entity(marked.entity)
        .insert(renders(&surface.image));
}

/// The render target of a camera that renders `image`.
fn renders(image: &Handle<Image>) -> RenderTarget {
    RenderTarget::Image(image.clone().into())
}

fn name_camera<C: Component>(
    mut commands: Commands,
    mut surface: ResMut<Surface>,
    camera: Single<Entity, With<C>>,
) {
    commands
        .entity(*camera)
        .insert((OnSurface, bevy::ui::IsDefaultUiCamera));
    surface.camera = Some(*camera);
}

/// Why a capture left no picture. [`image_to_file`] gives this; [`when_ready`] says
/// more ([`Missed`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// The picture came back and could not be written.
    Unsaved(String),
    /// No picture came back: the host stopped stepping, or the deadline passed.
    Incomplete,
}

/// Why a capture taken by [`when_ready`] left no picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Missed {
    /// No picture was asked for: the render pipelines and the caller's condition did
    /// not both hold on two consecutive frames before the capture ended.
    Unready(Ended),
    /// The picture was asked for and did not come back.
    Incomplete(Ended),
    /// The picture came back and could not be written.
    Unsaved(String),
}

/// What ended a capture before it had a picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ended {
    /// The host stopped stepping: the caller's step said it cannot go on.
    Stopped,
    /// The caller's deadline passed.
    Deadline,
}

/// What a capture's observer may still do, and what it did.
enum Request {
    Waiting,
    Written(Result<(), String>),
    /// The caller has its answer, written or given up, so nothing that arrives
    /// later is written.
    Ended,
}

/// Capture `image` to `path` once the table is ready, returning once the file is
/// written: the one loop for the agent seat's `capture` and for a capture a game takes
/// with no seat. `step` runs one update of the caller's host, on the caller's clock, and
/// says whether the host can go on; at least two milliseconds pass between steps that
/// find the table unready, and between steps that wait for the picture.
///
/// The picture is asked for only after the render pipelines ([`Pipelines`]) and
/// `ready`, the caller's own condition, have both held after two consecutive steps,
/// since the pipelines' finding trails the renderer by a frame. Readiness and the
/// picture share `deadline`, which is checked before each readiness step. What the
/// caller waits for, and on what clock, stays its own.
///
/// Once the capture has returned, a picture that arrives for it is not written, so one
/// that ends [`Missed::Incomplete`] leaves no file behind later. Its request stays in
/// the world until Bevy has handed that picture back, because Bevy marks the request
/// when it does.
pub fn when_ready(
    app: &mut App,
    image: Handle<Image>,
    path: &Path,
    deadline: Instant,
    mut ready: impl FnMut(&World) -> bool,
    mut step: impl FnMut(&mut App) -> bool,
) -> Result<(), Missed> {
    let pipelines = Pipelines::of(app);
    let mut held = 0;
    while held < 2 {
        if Instant::now() >= deadline {
            return Err(Missed::Unready(Ended::Deadline));
        }
        if !step(app) {
            return Err(Missed::Unready(Ended::Stopped));
        }
        if pipelines.ready() && ready(app.world()) {
            held += 1;
        } else {
            held = 0;
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    ask(app, image, path, deadline, step)
}

/// Capture `image` to `path`, returning once the file is written. `step` runs one
/// update of the caller's host, on the caller's clock, and says whether the host
/// can go on; at least two milliseconds pass between steps. The caller decides
/// what must be ready before it asks.
///
/// Once the capture has returned, a picture that arrives for it is not written.
/// So one that ends [`Failure::Incomplete`] leaves no file behind later. Its
/// request stays in the world until Bevy has handed that picture back, because
/// Bevy marks the request when it does. This differs from Arkham on purpose: there a
/// capture given up as incomplete still wrote its file when the picture came later,
/// and a picture handed back twice was written twice.
#[deprecated(
    note = "use `when_ready`, which waits for the render pipelines and says whether the host stopped or the deadline passed"
)]
pub fn image_to_file(
    app: &mut App,
    image: Handle<Image>,
    path: &Path,
    deadline: Instant,
    step: impl FnMut(&mut App) -> bool,
) -> Result<(), Failure> {
    ask(app, image, path, deadline, step).map_err(|missed| match missed {
        Missed::Unsaved(error) => Failure::Unsaved(error),
        Missed::Unready(_) | Missed::Incomplete(_) => Failure::Incomplete,
    })
}

/// Ask for the picture and step until it is written, the host stops or the deadline
/// passes.
fn ask(
    app: &mut App,
    image: Handle<Image>,
    path: &Path,
    deadline: Instant,
    mut step: impl FnMut(&mut App) -> bool,
) -> Result<(), Missed> {
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
                Request::Written(result) => Some(result.clone().map_err(Missed::Unsaved)),
                _ if !proceed => Some(Err(Missed::Incomplete(Ended::Stopped))),
                _ if Instant::now() >= deadline => Some(Err(Missed::Incomplete(Ended::Deadline))),
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
