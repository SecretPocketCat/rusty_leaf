use crate::{
    board::Board,
    mouse::CursorWorldPosition,
    piece::{spawn_piece, Piece, PieceFields},
    tile_placement::{BOARD_SIZE, TILE_SIZE},
    GameState,
};
use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin, RegisterInspectable};
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

#[derive(Component, Debug, Default, Inspectable)]
pub struct TileCoords {
    pub tile_coords: Option<UVec2>,
}

pub fn get_tile_coords_from_world(world_coords: Vec2, tile_size: UVec2) -> Option<UVec2> {
    let max_i = BOARD_SIZE as f32;
    let base_coords = world_coords.div(60.).round().add(Vec2::splat(4.));
    let coords = Vec2::new(base_coords.x - 1., max_i - 1. - base_coords.y.abs());
    let tile_size = Vec2::new(tile_size.x as f32, tile_size.y as f32);

    info!("{base_coords}, {coords}, {tile_size}");
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
    )
}

fn update_tile_coords(
    cursor_pos: Res<CursorWorldPosition>,
    mut dragged_query: Query<(&mut TileCoords, &Transform, &Interactable), With<Dragged>>,
) {
    if cursor_pos.is_changed() {
        if let Ok((mut coords, interactable_t, interactable)) = dragged_query.get_single_mut() {
            let tile_size = interactable.bounding_box.0.abs().div(TILE_SIZE / 2.);
            let tile_size = UVec2::new(tile_size.x as u32, tile_size.y as u32);
            let dragged_tile_coords = get_tile_coords_from_world(
                interactable_t.translation.truncate()
                    + Vec2::new(
                        -interactable.bounding_box.0.x.abs() + 1.5 * TILE_SIZE,
                        interactable.bounding_box.0.y.abs() - 0.5 * TILE_SIZE,
                    ),
                tile_size,
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