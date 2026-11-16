//! Text rows rendered as ink on flat world-space planes.
//!
//! Extracted from Arkham's `gathering_tabletop/printed.rs` at
//! d0212e5eecf21c358472a62c93c8992607497263. Consumers own text bundles,
//! typography, placement, names, picking, actions and scene lifetime.
//!
//! Keep an [`Atlas`] across scene rebuilds. [`Atlas::render`] lays the caller's
//! rows into one transparent image, through a `Camera2d` that keeps Bevy's
//! defaults apart from the supplied order and a transparent clear color. The
//! image is reused while its pixel dimensions stay the same, and replaced when
//! the width, row height or row count changes. [`Rendered::spawn`] maps one row
//! onto an unlit, blended XZ plane at the caller's exact transform. This is not a
//! typography framework, and it installs no plugin.
//!
//! On each rebuild the caller despawns the returned camera, root and planes; the
//! root owns its descendants. Each consumer keeps its own rendered inspection,
//! including live text changes.
use bevy::{
    camera::RenderTarget, mesh::VertexAttributeValues, prelude::*,
    render::render_resource::TextureFormat,
};

/// Pixel dimensions of each row and the offscreen camera's ordering.
/// The UI scale compensates row geometry; divide the caller's font sizes by
/// that same scale before supplying the text bundles.
pub struct Settings {
    pub width: u32,
    pub row_height: u32,
    pub camera_order: isize,
    pub ui_scale: f32,
}

/// Retains only the atlas image across caller-owned scene rebuilds.
#[derive(Default)]
pub struct Atlas(Option<(UVec2, Handle<Image>)>);

/// One rendered row set. Despawn `camera`, `root` (including its descendants)
/// and the spawned planes when rebuilding the caller's scene. Keep the `Atlas`
/// to reuse its image when the pixel dimensions have not changed.
pub struct Rendered {
    pub camera: Entity,
    pub root: Entity,
    material: Handle<StandardMaterial>,
    aspect: f32,
    rows: usize,
}

impl Atlas {
    /// Render nonempty, caller-owned bundles into centered, equal-height rows.
    ///
    /// Dimensions and UI scale must be positive (and the scale finite); the
    /// atlas height must fit in a `u32`. Invalid settings panic before spawning.
    /// The caller supplies Bevy's asset, UI and rendering plugins. No plugin or
    /// camera tracking system is installed by this module.
    pub fn render<B: Bundle>(
        &mut self,
        commands: &mut Commands,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
        settings: Settings,
        rows: Vec<B>,
    ) -> Rendered {
        assert!(settings.width > 0 && settings.row_height > 0);
        assert!(settings.ui_scale.is_finite() && settings.ui_scale > 0.);
        assert!(!rows.is_empty(), "a printed atlas needs at least one row");
        let row_count = rows.len();
        let height = u32::try_from(row_count)
            .ok()
            .and_then(|count| settings.row_height.checked_mul(count))
            .expect("printed atlas height fits in u32");
        let size = UVec2::new(settings.width, height);
        if self.0.as_ref().is_none_or(|(old, _)| *old != size) {
            self.0 = Some((
                size,
                images.add(Image::new_target_texture(
                    size.x,
                    size.y,
                    TextureFormat::Bgra8UnormSrgb,
                    None,
                )),
            ));
        }
        let image = self.0.as_ref().expect("atlas image initialized").1.clone();
        let camera = commands
            .spawn((
                Camera2d,
                Camera {
                    order: settings.camera_order,
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    ..default()
                },
                RenderTarget::Image(image.clone().into()),
            ))
            .id();
        let root = commands
            .spawn((
                UiTargetCamera(camera),
                Node {
                    width: percent(100),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|root| {
                for text in rows {
                    root.spawn((
                        Node {
                            width: px(settings.width as f32 / settings.ui_scale),
                            height: px(settings.row_height as f32 / settings.ui_scale),
                            flex_shrink: 0.,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|row| {
                        row.spawn((text, Pickable::IGNORE));
                    });
                }
            })
            .id();
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(image),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        Rendered {
            camera,
            root,
            material,
            aspect: settings.width as f32 / settings.row_height as f32,
            rows: row_count,
        }
    }
}

impl Rendered {
    /// Spawn one row on a flat XZ plane, with its pixel aspect ratio and the
    /// caller's exact transform. Height is the plane's local Z extent.
    ///
    /// `row` must index this render's row set; `height` must be finite and
    /// positive. Invalid inputs panic before spawning. Planes are non-pickable
    /// by default; consumers may replace that component to attach an action.
    pub fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        row: usize,
        height: f32,
        transform: Transform,
    ) -> Entity {
        assert!(row < self.rows, "printed row is outside the atlas");
        assert!(height.is_finite() && height > 0.);
        let mut mesh = Plane3d::default()
            .mesh()
            .size(height * self.aspect, height)
            .build();
        if let Some(VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            for uv in uvs {
                uv[1] = (row as f32 + uv[1]) / self.rows as f32;
            }
        }
        commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(self.material.clone()),
                transform,
                Pickable::IGNORE,
            ))
            .id()
    }
}
