use crate::{
    board::Board,
    coords::{get_world_coords_from_tile, TileCoords},
    piece::Piece,
    render::ZIndex,
    tile_placement::{Pieces, BOARD_SIZE_PX},
};
use bevy::prelude::*;
use bevy_interact_2d::{drag::Dragged, Group, Interactable};
use bevy_tweening::{Animator, AnimatorState};

pub struct DragPlugin;
impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Last, limit_drag_count)
            .add_system_to_stage(CoreStage::Last, disable_drag_during_tween)
            .add_system(drag_piece)
            .add_system(raise_z_index)
            .add_system(restore_z_index)
            .add_system(process_movers);
    }
}

#[derive(Component)]
pub struct DragZOffset {
    original_z: f32,
    original_z_index: Option<ZIndex>,
}

#[derive(Clone, Copy)]
pub enum DragGroup {
    Piece = 1,
    Card,
    Cauldron,
    Fire,
    Grid,
    GridPieces,
    GridSection,
}

impl From<DragGroup> for Group {
    fn from(g: DragGroup) -> Self {
        Group(g as u8)
    }
}

#[derive(Component)]
pub struct Mover {
    pub moved_e: Entity,
}

fn drag_piece(
    _cmd: Commands,
    mouse_input: Res<Input<MouseButton>>,
    board: Res<Board>,
    pieces: Res<Pieces>,
    dragged_query: Query<(Entity, &Piece, &TileCoords), (With<Dragged>, Changed<TileCoords>)>,
) {
    if mouse_input.pressed(MouseButton::Left) {
        if let Ok((_, piece, coords)) = dragged_query.get_single() {
            if let Some(coords) = coords.tile_coords {
                if board
                    .can_place_piece(
                        coords.x as usize,
                        coords.y as usize,
                        pieces.pieces[piece.0].get_fields(),
                    )
                    .is_ok()
                {
                    // todo: colour outline or smt.
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

fn raise_z_index(
    mut cmd: Commands,
    dragged_q: Query<(Entity, &Transform, Option<&ZIndex>), Added<Dragged>>,
) {
    for (e, dragged_t, dragged_z) in dragged_q.iter() {
        let mut cmd_e = cmd.entity(e);
        cmd_e.insert(DragZOffset {
            original_z: dragged_t.translation.z,
            original_z_index: dragged_z.cloned(),
        });
        cmd_e.insert(ZIndex::Dragged);
    }
}

fn restore_z_index(
    mut cmd: Commands,
    drag_removed: RemovedComponents<Dragged>,
    mut dragged_q: Query<(&DragZOffset, &mut Transform, &mut ZIndex)>,
) {
    for e in drag_removed.iter() {
        if let Ok((z_offset, mut t, mut dragged_z)) = dragged_q.get_mut(e) {
            // todo: might wanna put this on a timer or till animators are done...
            if let Some(z_index) = z_offset.original_z_index {
                *dragged_z = z_index;
            } else {
                t.translation.z = z_offset.original_z;
                cmd.entity(e).remove::<ZIndex>();
            }

            cmd.entity(e).remove::<DragZOffset>();
        }
    }
}

fn disable_drag_during_tween(
    mut cmd: Commands,
    dragged_q: Query<(Entity, &Animator<Transform>), With<Dragged>>,
) {
    for (e, anim) in dragged_q.iter() {
        if anim.tweenable().progress() < 1. {
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
