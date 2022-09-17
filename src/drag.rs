use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_interact_2d::{
    drag::{Draggable, Dragged, DropStrategy},
    Group, Interactable,
};

use crate::{
    board::{Board, BoardClear},
    coords::{get_world_coords_from_tile, TileCoords},
    piece::Piece,
    render::WINDOW_SIZE,
    tile_placement::{Pieces, BOARD_SHIFT, BOARD_SIZE_PX},
};

// todo: cancel tween if clicking on a draggable entity
pub struct DragPlugin;
impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Last, limit_drag_count)
            .add_system(drag_piece)
            .add_system(process_movers);
    }
}

#[derive(Clone, Copy)]
pub enum DragGroup {
    Piece = 1,
    Card,
}

impl Into<Group> for DragGroup {
    fn into(self) -> Group {
        Group(self as u8)
    }
}

#[derive(Component)]
pub struct Mover {
    pub moved_e: Entity,
}

fn drag_piece(
    mut cmd: Commands,
    mouse_input: Res<Input<MouseButton>>,
    board: Res<Board>,
    pieces: Res<Pieces>,
    dragged_query: Query<(Entity, &Piece, &TileCoords), (With<Dragged>, Changed<TileCoords>)>,
) {
    if mouse_input.pressed(MouseButton::Left) {
        if let Ok((_, piece, coords)) = dragged_query.get_single() {
            if let Some(coords) = coords.tile_coords {
                if let Ok(_) = board.can_place_piece(
                    coords.x as usize,
                    coords.y as usize,
                    pieces.pieces[piece.0].get_fields(),
                ) {
                    // todo: colour outline or smt.
                    info!("can place!");
                }
            }
        }
    }
}

fn limit_drag_count(mut cmd: Commands, dragged_query: Query<Entity, With<Dragged>>) {
    if dragged_query.iter().len() > 1 {
        for e in dragged_query.iter().skip(1) {
            cmd.entity(e).remove::<Dragged>();
        }
    }
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
                        interactable.bounding_box.0.x.abs(),
                        -interactable.bounding_box.0.y.abs(),
                    ))
                .extend(z)
            } else {
                mover_t.translation
            };
        }
    }
}
