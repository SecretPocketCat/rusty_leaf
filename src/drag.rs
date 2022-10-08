use std::ops::Sub;

use crate::{
    board::Board,
    coords::{get_world_coords_from_tile, TileCoords},
    interaction::{Interactable, InteractionEv},
    mouse::CursorWorldPosition,
    piece::Piece,
    render::{ViewScale, ZIndex},
    tile_placement::{Pieces, BOARD_SIZE_PX},
};
use bevy::prelude::*;
use bevy_extensions::{asymptotic_smoothing_with_delta_time, inverse_lerp_clamped};
use bevy_tweening::{Animator, AnimatorState};
use web_sys::console::info;

pub struct DragPlugin;
impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(on_drag_start)
            .add_system(on_drag_end.after(on_drag_start))
            .add_system_to_stage(CoreStage::Last, drag);
    }
}

#[derive(Component)]
pub struct Draggable {
    pub offset: bool,
}

#[derive(Component)]
pub struct Dragged {
    pub origin: Vec2,
    original_z: f32,
    original_z_index: Option<ZIndex>,
    offset: Vec2,
}

fn on_drag_start(
    mut cmd: Commands,
    cursor: Res<CursorWorldPosition>,
    mut evr: EventReader<InteractionEv>,
    dragged_q: Query<(&Transform, Option<&ZIndex>)>,
) {
    for ev in evr.iter() {
        if let InteractionEv::DragStart(drag_data) = ev {
            if let Ok((t, z)) = dragged_q.get(drag_data.e) {
                cmd.entity(drag_data.e)
                    .insert(Dragged {
                        origin: drag_data.origin,
                        original_z: t.translation.z,
                        original_z_index: z.cloned(),
                        offset: t.translation.truncate() - cursor.position,
                    })
                    .insert(ZIndex::Dragged);

                break;
            }
        }
    }
}

fn on_drag_end(
    mut cmd: Commands,
    mut evr: EventReader<InteractionEv>,
    mut dragged_q: Query<(&Dragged, &mut Transform, &mut ZIndex)>,
) {
    for ev in evr.iter() {
        if let InteractionEv::DragEnd(drag_data) = ev {
            if let Ok((dragged, mut t, mut dragged_z)) = dragged_q.get_mut(drag_data.e) {
                // todo: delay this to prevent z fighting on drag end
                if let Some(z_index) = dragged.original_z_index {
                    *dragged_z = z_index;
                } else {
                    t.translation.z = dragged.original_z;
                    cmd.entity(drag_data.e).remove::<ZIndex>();
                }

                cmd.entity(drag_data.e).remove::<Dragged>();

                break;
            }
        }
    }
}

fn drag(
    mut dragged_q: Query<(&mut Dragged, &mut Transform, &Interactable, &Draggable)>,
    cursor: Res<CursorWorldPosition>,
    time: Res<Time>,
) {
    for (mut dragged, mut t, interactable, draggable) in dragged_q.iter_mut() {
        if cursor.is_changed() {
            if draggable.offset {
                let offset_t =
                    inverse_lerp_clamped(0., 50. * time.delta_seconds(), cursor.delta.length())
                        + 0.5;

                dragged.offset = asymptotic_smoothing_with_delta_time(
                    dragged.offset,
                    Vec2::new(interactable.bounds.max.x, interactable.bounds.min.y),
                    0.2 * offset_t,
                    time.delta_seconds(),
                );
            }

            t.translation = (cursor.position + dragged.offset).extend(t.translation.z);
        }
    }
}
