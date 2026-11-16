//! Keeps a caller's UI node over a world anchor.
//!
//! Generalized for issue #35 from `count_label`, itself extracted from Arkham's
//! `tabletop_counters` at d7a6f2ea46fbf97719cc352ee8b94270857b9bcd, with
//! Netrunner's board labels at df604944d3c7ad96b786df5edf6c0959e79733f2.
//!
//! The caller spawns the node and owns everything on it: its text, font,
//! colour, padding, accessibility name, pickability, stacking and lifetime. It
//! gives the node `PositionType::Absolute` and adds a [`LabelAnchor`]. Each
//! frame this module writes only the node's `left` and `top` and its
//! `Visibility`, and removes a node whose anchor entity has lost its
//! `Transform`. A margin the caller sets still shifts the node.
use bevy::{camera::CameraUpdateSystems, prelude::*, ui::UiSystems};

pub struct LabelAnchorPlugin;

impl Plugin for LabelAnchorPlugin {
    fn build(&self, app: &mut App) {
        // After the camera has moved and before layout reads the position, as
        // Bevy's `world_to_viewport` asks.
        app.add_systems(
            PostUpdate,
            position
                .after(TransformSystems::Propagate)
                .after(CameraUpdateSystems)
                .before(UiSystems::Layout),
        );
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
    /// by its scale but not rotated, and a parent is not followed. The node is
    /// removed once the entity has no `Transform`.
    Entity { entity: Entity, offset: Vec3 },
    /// A fixed world point.
    Point(Vec3),
}

/// Which part of the node's top edge sits at the anchor's projected point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    /// The middle, by the node's measured width.
    #[default]
    Centre,
    /// The left end.
    LeftEdge,
}

impl LabelAnchor {
    pub fn entity(entity: Entity, offset: Vec3) -> Self {
        Self::to(Anchor::Entity { entity, offset })
    }

    pub fn point(at: Vec3) -> Self {
        Self::to(Anchor::Point(at))
    }

    pub fn within_viewport(self) -> Self {
        Self {
            within_viewport: true,
            ..self
        }
    }

    pub fn left_edge(self) -> Self {
        Self {
            align: Align::LeftEdge,
            ..self
        }
    }

    fn to(to: Anchor) -> Self {
        Self {
            to,
            align: Align::Centre,
            within_viewport: false,
        }
    }
}

// One 3D camera is required: with none, or several, nothing changes.
fn position(
    mut commands: Commands,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    transforms: Query<&Transform>,
    mut labels: Query<(
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
    for (entity, label, computed, mut node, mut visibility) in &mut labels {
        let at = match label.to {
            Anchor::Entity {
                entity: anchor,
                offset,
            } => {
                let Ok(at) = transforms.get(anchor) else {
                    commands.entity(entity).try_despawn();
                    continue;
                };
                at.translation + offset * at.scale
            }
            Anchor::Point(at) => at,
        };
        if let Ok(point) = camera.world_to_viewport(camera_at, at)
            && (!label.within_viewport
                || camera
                    .logical_viewport_rect()
                    .is_some_and(|rect| rect.contains(point)))
        {
            node.left = px(match label.align {
                Align::Centre => point.x - computed.size().x * computed.inverse_scale_factor / 2.,
                Align::LeftEdge => point.x,
            });
            node.top = px(point.y);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
