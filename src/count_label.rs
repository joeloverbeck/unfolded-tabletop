//! Numeric UI labels that follow a root token's position and scale.
//!
//! Extracted from Arkham's `tabletop_counters` at
//! d7a6f2ea46fbf97719cc352ee8b94270857b9bcd. Consumers own token construction,
//! zero-count policy, offsets, fonts, colors and accessibility names.
//!
//! Kept while games move to [`crate::label_anchor`], which tracks these labels
//! and any other node a caller owns (issue #35). New code spawns its own node
//! with a [`LabelAnchor`] instead of calling [`spawn`].
//!
//! Since issue #35, [`spawn`] adds a [`LabelAnchor`] to the count it makes, and
//! [`CountLabelPlugin`] installs [`LabelAnchorPlugin`].
use crate::label_anchor::{LabelAnchor, LabelAnchorPlugin};
use bevy::prelude::*;

/// Installs [`LabelAnchorPlugin`], which a game may also install itself, before
/// or after this.
pub struct CountLabelPlugin;

impl Plugin for CountLabelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LabelAnchorPlugin);
    }
}

/// Which token a count was spawned over. It moves nothing: the count's
/// [`LabelAnchor`] does.
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
            LabelAnchor::entity(token, offset),
            Pickable::IGNORE,
        ))
        .id()
}
