use crate::{
    board::Board,
    coords::{get_world_coords_from_tile, TileCoords},
    interaction::Interactable,
    mouse::CursorWorldPosition,
    piece::Piece,
    render::{ViewScale, ZIndex},
    tile_placement::{Pieces, BOARD_SIZE_PX},
};
use bevy::prelude::*;
use bevy_tweening::{Animator, AnimatorState};

pub struct MoverPlugin;
impl Plugin for MoverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(process_movers);
    }
}

#[derive(Component)]
pub struct Mover {
    pub moved_e: Entity,
}

fn process_movers(
    mover_q: Query<(&Mover, &Transform, &TileCoords, &Interactable)>,
    mut moved_q: Query<&mut Transform, Without<Mover>>,
) {
    for (mover, mover_t, coords, interactable) in mover_q.iter() {
        if let Ok(mut t) = moved_q.get_mut(mover.moved_e) {
            let z = t.translation.z;
            t.translation = if let Some(pos) = coords.tile_coords {
                (get_world_coords_from_tile(pos)
                    + Vec2::new(-BOARD_SIZE_PX / 2., BOARD_SIZE_PX / 2.)
                    + Vec2::new(
                        interactable.bounds.width() / 2.,
                        -interactable.bounds.height() / 2.,
                    ))
                .extend(z)
            } else {
                mover_t.translation
            };
        }
    }
}
