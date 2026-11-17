//! Keeps a caller's UI node over a world anchor.
//!
//! Generalized for issue #35 from `count_label`, itself extracted from Arkham's
//! `tabletop_counters` at d7a6f2ea46fbf97719cc352ee8b94270857b9bcd, with
//! Netrunner's board labels at df604944d3c7ad96b786df5edf6c0959e79733f2.
//!
//! The caller spawns the node and owns everything on it but its position and
//! visibility: its text, font, color, size, margin, padding, accessibility
//! name, pickability, stacking and lifetime. It gives the node
//! `PositionType::Absolute` and adds a [`LabelAnchor`]. Each frame this module
//! writes the node's `left`, `top` and `Visibility`, and removes a node whose
//! anchor entity has lost its `Transform`. A caller that hides a node itself
//! uses `Display::None` or despawns it, since `Visibility` is rewritten.
//!
//! Install [`LabelAnchorPlugin`]. Its system runs in `PostUpdate` after transform
//! propagation and the camera's update, so a camera or anchor moved this frame is
//! projected where it now is. The node is hidden when projection fails. Tracking
//! needs exactly one `Camera3d`: with none, or several, it changes nothing. It
//! makes no promise about several views, or about offsets that rotate with a
//! parent. When a count updates and whether a zero shows stay with the caller.
use bevy::{
    camera::{CameraUpdateSystems, visibility::VisibilitySystems},
    prelude::*,
    ui::UiSystems,
};

/// Installs the tracking system once, however many times it is added, so several
/// of a game's own plugins may each add it.
pub struct LabelAnchorPlugin;

#[derive(Resource)]
struct Installed;

impl Plugin for LabelAnchorPlugin {
    fn build(&self, app: &mut App) {
        if app.world().contains_resource::<Installed>() {
            return;
        }
        app.insert_resource(Installed);
        // After the camera and anchors have moved, as Bevy's `world_to_viewport`
        // asks, and before visibility and layout read what it writes, so a node
        // is drawn where it belongs in the frame its anchor moves.
        app.add_systems(
            PostUpdate,
            position
                .after(TransformSystems::Propagate)
                .after(CameraUpdateSystems)
                .before(VisibilitySystems::VisibilityPropagate)
                .before(UiSystems::Layout),
        );
    }

    fn is_unique(&self) -> bool {
        false
    }
}

/// Where a node is kept, and how.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct LabelAnchor {
    pub to: Anchor,
    pub align: Align,
    /// Hide the node while its point falls outside the camera's logical
    /// viewport rectangle, as well as when projection fails.
    pub within_viewport: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Anchor {
    /// An entity's own `Transform`, as for a root token: the offset is scaled
    /// by its scale but not rotated, and its parent's transform is not read.
    /// The node is removed once the entity has no `Transform`.
    Entity { entity: Entity, offset: Vec3 },
    /// A fixed world point.
    Point(Vec3),
}

/// Which part of the node's top edge sits at the anchor's projected point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    /// The middle, by the node's measured width.
    #[default]
    Center,
    /// The left end.
    LeftEdge,
}

impl LabelAnchor {
    pub fn entity(entity: Entity, offset: Vec3) -> Self {
        Self::new(Anchor::Entity { entity, offset })
    }

    pub fn point(at: Vec3) -> Self {
        Self::new(Anchor::Point(at))
    }

    pub fn left_edge(self) -> Self {
        Self {
            align: Align::LeftEdge,
            ..self
        }
    }

    pub fn within_viewport(self) -> Self {
        Self {
            within_viewport: true,
            ..self
        }
    }

    fn new(to: Anchor) -> Self {
        Self {
            to,
            align: Align::Center,
            within_viewport: false,
        }
    }
}

// One 3D camera is required: with none, or several, nothing changes.
fn position(
    mut commands: Commands,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    transforms: Query<&Transform>,
    mut nodes: Query<(
        Entity,
        &LabelAnchor,
        &ComputedNode,
        &mut Node,
        &mut Visibility,
    )>,
) {
    let Ok((camera, camera_at)) = camera.single() else {
        return;
    };
    for (entity, anchor, computed, mut node, mut visibility) in &mut nodes {
        let world = match anchor.to {
            Anchor::Entity {
                entity: anchored,
                offset,
            } => {
                let Ok(transform) = transforms.get(anchored) else {
                    commands.entity(entity).try_despawn();
                    continue;
                };
                transform.translation + offset * transform.scale
            }
            Anchor::Point(at) => at,
        };
        if let Ok(point) = camera.world_to_viewport(camera_at, world)
            && (!anchor.within_viewport
                || camera
                    .logical_viewport_rect()
                    .is_some_and(|rect| rect.contains(point)))
        {
            node.left = px(match anchor.align {
                Align::Center => point.x - computed.size().x * computed.inverse_scale_factor / 2.,
                Align::LeftEdge => point.x,
            });
            node.top = px(point.y);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
