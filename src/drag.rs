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
use bevy_tweening::{Animator, AnimatorState};

pub struct DragPlugin;
impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(on_drag_start)
            .add_system(on_drag_end.after(on_drag_start))
            .add_system_to_stage(CoreStage::Last, drag);
    }
}

#[derive(Component)]
pub struct Draggable;

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
                        offset: t.translation.truncate() - cursor.0,
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

fn drag(mut dragged_q: Query<(&Dragged, &mut Transform)>, cursor: Res<CursorWorldPosition>) {
    for (dragged, mut t) in dragged_q.iter_mut() {
        if cursor.is_changed() {
            t.translation = (cursor.0 + dragged.offset).extend(t.translation.z);
        }
    }
}
