use bevy::prelude::*;

use crate::mouse::CursorWorldPosition;

pub struct TilePlacementPlugin;

impl Plugin for TilePlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_tile_coords);
    }
}

fn update_tile_coords(cursor_pos: Res<CursorWorldPosition>) {
    info!("{}", cursor_pos.0);
}
