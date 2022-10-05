use crate::{
    drag::Draggable,
    interaction::{Interactable, InteractionEv, InteractionGroup, InteractionState},
    tween::{get_relative_sprite_color_anim, get_relative_spritesheet_color_anim},
};
use bevy::{prelude::*, utils::HashSet};

pub struct HighlightPlugin;
impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(highlight_interactable_on_drag)
            .add_system(highlight_draggable_on_hover);
    }
}

#[derive(Component)]
pub struct Highligtable {
    pub sprite_e: Option<Entity>,
    pub hightlight_color: Color,
    pub hover_color: Color,
    pub normal_color: Color,
    pub drag_groups: Vec<InteractionGroup>,
}

fn highlight_interactable_on_drag(
    mut evr: EventReader<InteractionEv>,
    mut cmd: Commands,
    interactable_q: Query<&Interactable>,
    highlightable_q: Query<(Entity, &Highligtable)>,
    sprite_q: Query<&Sprite>,
) {
    for ev in evr.iter() {
        if let Some((e, start)) = match ev {
            InteractionEv::DragStart(data) => Some((data.e, true)),
            InteractionEv::DragEnd(data) => Some((data.e, false)),
            _ => None,
        } {
            for (highlightable_e, highlightable) in
                highlightable_q.iter().filter(|(e2, ..)| *e2 != e)
            {
                if let Ok(interactable) = interactable_q.get(e) {
                    tween_outline_with_interaction_group_check(
                        &mut cmd,
                        highlightable,
                        highlightable_e,
                        &interactable.group,
                        &sprite_q,
                        if start {
                            highlightable.hightlight_color
                        } else {
                            highlightable.normal_color
                        },
                    );
                }
            }
        }
    }
}

fn highlight_draggable_on_hover(
    mut evr: EventReader<InteractionEv>,
    mut cmd: Commands,
    state: Res<InteractionState>,
    sprite_q: Query<&Sprite>,
    highlightable_q: Query<(Entity, &Highligtable, Option<&Draggable>)>,
    interactable_q: Query<&Interactable>,
) {
    for ev in evr.iter() {
        if let Some((e, start)) = match ev {
            InteractionEv::HoverStart(data) => Some((data.e, true)),
            InteractionEv::HoverEnd(data) => Some((data.e, false)),
            _ => None,
        } {
            if let Ok((highlightable_e, highlightable, draggable)) = highlightable_q.get(e) {
                if let Some(dragged_e) = state.dragged_e {
                    // already dragging => hover with a dragged item
                    if let Ok(interactable) = interactable_q.get(dragged_e) {
                        tween_outline_with_interaction_group_check(
                            &mut cmd,
                            highlightable,
                            highlightable_e,
                            &interactable.group,
                            &sprite_q,
                            if start {
                                highlightable.hover_color
                            } else {
                                highlightable.hightlight_color
                            },
                        );
                    }
                } else if draggable.is_some() {
                    // not dragging - hover over a draggable card
                    tween_outline(
                        &mut cmd,
                        highlightable,
                        highlightable_e,
                        &sprite_q,
                        if start {
                            highlightable.hover_color
                        } else {
                            highlightable.normal_color
                        },
                    );
                }
            }
        }
    }
}

fn tween_outline_with_interaction_group_check(
    cmd: &mut Commands,
    highlightable: &Highligtable,
    highlightable_e: Entity,
    interactable_group: &InteractionGroup,
    sprite_q: &Query<&Sprite>,
    color: Color,
) {
    if highlightable.drag_groups.contains(interactable_group) {
        tween_outline(cmd, highlightable, highlightable_e, sprite_q, color);
    }
}

fn tween_outline(
    cmd: &mut Commands,
    highlightable: &Highligtable,
    highlightable_e: Entity,
    sprite_q: &Query<&Sprite>,
    color: Color,
) {
    let e = highlightable.sprite_e.unwrap_or(highlightable_e);
    if sprite_q.contains(e) {
        // sprite
        cmd.entity(e)
            .insert(get_relative_sprite_color_anim(color, 220, None));
    } else {
        // spritesheet
        cmd.entity(e)
            .insert(get_relative_spritesheet_color_anim(color, 220, None));
    }
}
