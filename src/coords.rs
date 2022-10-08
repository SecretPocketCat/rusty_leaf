use crate::{
    board::Board,
    drag::Dragged,
    interaction::Interactable,
    mouse::CursorWorldPosition,
    piece::Piece,
    tile_placement::{Pieces, BOARD_SHIFT, BOARD_SIZE, TILE_SIZE},
};
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

use std::ops::{Add, Div};

pub struct CoordsPlugin;
impl Plugin for CoordsPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_system(log_coords)
            .add_system(update_tile_coords);
    }
}

#[derive(Component, Debug, Default, Inspectable)]
pub struct TileCoords {
    pub tile_coords: Option<UVec2>,
}

pub fn get_tile_coords_from_world(world_coords: Vec2, tile_size: UVec2) -> Option<UVec2> {
    let max_i = BOARD_SIZE as f32;
    let base_coords = world_coords.div(TILE_SIZE).round();
    let coords = Vec2::new(base_coords.x - 1., max_i - 1. - base_coords.y.abs());
    let tile_size = Vec2::new(tile_size.x as f32, tile_size.y as f32);

    if coords.min_element() >= 0.
        && coords.max_element() < max_i
        && base_coords.y >= 0.
        && base_coords.y - tile_size.y + 1. >= 0.
        && coords.x + tile_size.x - 1. < max_i
    {
        Some(UVec2::new(coords.x as u32, coords.y as u32))
    } else {
        None
    }
}

pub fn get_world_coords_from_tile(tile_coords: UVec2) -> Vec2 {
    Vec2::new(
        tile_coords.x as f32 * TILE_SIZE,
        tile_coords.y as f32 * -TILE_SIZE,
    ) + BOARD_SHIFT.truncate()
}

fn update_tile_coords(
    cursor_pos: Res<CursorWorldPosition>,
    mut dragged_query: Query<(&mut TileCoords, &Piece, &Transform, &Interactable), With<Dragged>>,
    board: Res<Board>,
    pieces: Res<Pieces>,
) {
    if cursor_pos.is_changed() {
        if let Ok((mut coords, piece, interactable_t, interactable)) =
            dragged_query.get_single_mut()
        {
            let tile_size = interactable.bounds.size().div(TILE_SIZE);
            let tile_size = UVec2::new(tile_size.x as u32, tile_size.y as u32);
            let mut dragged_tile_coords = get_tile_coords_from_world(
                interactable_t.translation.truncate()
                    // todo: what's up with this magic offset?
                    + Vec2::new(
                        -interactable.bounds.width() / 2. + 5.5 * TILE_SIZE,
                        interactable.bounds.height() / 2. + 3.5 * TILE_SIZE,
                    )
                    + -BOARD_SHIFT.truncate(),
                tile_size,
            );

            if let Some(dragged_coords) = dragged_tile_coords {
                let piece = &pieces.pieces[piece.0];
                if board
                    .can_place_piece(
                        dragged_coords.x as usize,
                        dragged_coords.y as usize,
                        piece.get_fields(),
                    )
                    .is_err()
                {
                    dragged_tile_coords = None;
                }
            }

            if coords.tile_coords.is_some() && dragged_tile_coords.is_none() {
                coords.tile_coords = dragged_tile_coords;
            } else {
                match coords.tile_coords {
                    Some(prev_coords) => match dragged_tile_coords {
                        Some(new_coords) => {
                            if prev_coords != new_coords {
                                coords.tile_coords = dragged_tile_coords;
                            }
                        }
                        None => coords.tile_coords = dragged_tile_coords,
                    },
                    None => coords.tile_coords = dragged_tile_coords,
                }
            }
        }
    }
}

fn log_coords(cursor_pos: Res<CursorWorldPosition>) {
    if cursor_pos.is_changed() {
        info!(
            "Cursor coords [{}, {}]",
            cursor_pos.position.x, cursor_pos.position.y
        );
    }
}
