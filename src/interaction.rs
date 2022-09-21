// use crate::{
//     drag::DragGroup,
//     render::COL_DARK,
//     tween::{get_relative_sprite_color_anim, get_relative_spritesheet_color_anim},
// };
// use bevy::prelude::*;
// use bevy_interact_2d::{
//     drag::{Draggable, Dragged},
//     Group, Interactable,
// };

// pub struct InteractionPlugin;
// impl Plugin for InteractionPlugin {
//     fn build(&self, app: &mut App) {
//         app // .add_system_to_stage(CoreStage::PostUpdate, disable_drag_during_tween)
//             .add_system(highlight_interactable);
//     }
// }

// fn handle_interaction_events(
//     cursor_pos: Res<CursorWorldPosition>,
//     mut dragged_query: Query<(&mut TileCoords, &Piece, &Transform, &Interactable), With<Dragged>>,
//     board: Res<Board>,
//     pieces: Res<Pieces>,
// ) {
//     if cursor_pos.is_changed() {}
// }
