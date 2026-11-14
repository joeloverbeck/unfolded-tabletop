//! Numeric UI labels that follow a root token's position and scale.
//!
//! Extracted from Arkham's `tabletop_counters` at
//! d7a6f2ea46fbf97719cc352ee8b94270857b9bcd. Consumers own token construction,
//! zero-count policy, offsets, fonts, colors and accessibility names.
use bevy::prelude::*;

pub struct CountLabelPlugin;

impl Plugin for CountLabelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, position_labels);
    }
}

#[derive(Component)]
pub struct Label {
    pub token: Entity,
    pub offset: Vec3,
}

/// Spawn a non-pickable, centered count. Insert consumer presentation components
/// on the returned entity. Counts are snapshots; consumers own their updates.
pub fn spawn(commands: &mut Commands, token: Entity, amount: u32, offset: Vec3) -> Entity {
    commands
        .spawn((
            Text::new(amount.to_string()),
            TextLayout::justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            Label { token, offset },
            Pickable::IGNORE,
        ))
        .id()
}

// As in the original consumer, tokens are roots: the offset is scaled but not
// rotated. A single 3D camera is required. No camera leaves labels unchanged.
fn position_labels(
    mut commands: Commands,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    transforms: Query<&Transform>,
    mut labels: Query<(Entity, &Label, &ComputedNode, &mut Node, &mut Visibility)>,
) {
    let Ok((camera, camera_at)) = camera.single() else {
        return;
    };
    for (entity, label, computed, mut node, mut visibility) in &mut labels {
        let Ok(at) = transforms.get(label.token) else {
            commands.entity(entity).despawn();
            continue;
        };
        if let Ok(point) =
            camera.world_to_viewport(camera_at, at.translation + label.offset * at.scale)
        {
            let width = computed.size().x * computed.inverse_scale_factor;
            node.left = px(point.x - width / 2.0);
            node.top = px(point.y);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
