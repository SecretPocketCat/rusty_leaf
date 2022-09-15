use crate::{
    board::Board,
    mouse::CursorWorldPosition,
    piece::{spawn_piece, Piece, PieceFields},
    tile_placement::{BOARD_SIZE, TILE_SIZE},
    GameState,
};
use bevy::prelude::*;
use bevy_inspector_egui::InspectorPlugin;
use bevy_interact_2d::{
    drag::{Draggable, Dragged},
    Interactable, InteractionState,
};
use bevy_prototype_lyon::prelude::*;
use iyes_loopless::prelude::*;
use rand::Rng;
use std::{
    collections::VecDeque,
    ops::{Add, Div, Sub},
};

pub struct CoordsPlugin;
impl Plugin for CoordsPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_system(log_coords)
            .add_system(update_tile_coords);
    }
}

#[derive(Component, Debug, Default)]
pub struct TileCoords {
    pub tile_coords: Option<UVec2>,
}

pub fn get_tile_coords_from_world(world_coords: Vec2) -> Option<UVec2> {
    let max_i = BOARD_SIZE as f32 - 1.;
    let base_coords = world_coords.div(60.).round().add(Vec2::splat(4.));
    let coords = Vec2::new(base_coords.x - 1., max_i - base_coords.y.abs());

    if coords.x >= 0. && coords.x <= max_i && coords.y >= 0. && coords.y <= max_i {
        Some(UVec2::new(coords.x as u32, coords.y as u32))
    } else {
        None
    }
}

pub fn get_world_coords_from_tile(tile_coords: UVec2) -> Vec2 {
    Vec2::new(
        tile_coords.x as f32 * TILE_SIZE,
        tile_coords.y as f32 * -TILE_SIZE,
    )
}

fn update_tile_coords(
    cursor_pos: Res<CursorWorldPosition>,
    mut dragged_query: Query<(&mut TileCoords, &Transform, &Interactable), With<Dragged>>,
) {
    if cursor_pos.is_changed() {
        if let Ok((mut coords, interactable_t, interactable)) = dragged_query.get_single_mut() {
            let dragged_tile_coords = get_tile_coords_from_world(
                interactable_t.translation.truncate()
                    + Vec2::new(
                        -interactable.bounding_box.0.x.abs() + 90.,
                        interactable.bounding_box.0.y.abs() - 30.,
                    ),
            );

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
        info!("Cursor coords [{}, {}]", cursor_pos.0.x, cursor_pos.0.y);
    }
}
