use crate::{
    drag::DragGroup,
    tween::{get_relative_sprite_color_anim, get_relative_spritesheet_color_anim},
};
use bevy::{prelude::*, utils::HashSet};
use bevy_interact_2d::{
    drag::{self, Draggable, Dragged},
    Group, Interactable, InteractionState,
};

pub struct HighlightPlugin;
impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HighlightEv>()
            .insert_resource(HighlightState::default())
            .add_system(send_hover_events)
            .add_system(send_drag_events)
            .add_system(highlight_interactable_on_drag)
            .add_system(highlight_draggable_on_hover);
    }
}

#[derive(Default)]
struct HighlightState {
    dragged_entities: HashSet<Entity>,
    hovered_entities: HashSet<Entity>,
}

struct HoverData {
    entity: Entity,
    draggable: bool,
}

struct DragData {
    entity: Entity,
}

enum HighlightEv {
    HoverStart(HoverData),
    HoverEnd(HoverData),
    DragStart(DragData),
    DragEnd(DragData),
}

#[derive(Component)]
pub struct Highligtable {
    pub sprite_e: Option<Entity>,
    pub hightlight_color: Color,
    pub hover_color: Color,
    pub normal_color: Color,
    pub drag_groups: Vec<DragGroup>,
}

fn send_hover_events(
    mut evw: EventWriter<HighlightEv>,
    mut state: ResMut<HighlightState>,
    interaction_state: Res<InteractionState>,
    highlightable_q: Query<(Entity, &Interactable, Option<&Draggable>), With<Highligtable>>,
) {
    for (highlightable_e, interactable, draggable) in highlightable_q.iter() {
        if let Some(g) = interactable.groups.first() {
            if interaction_state
                .get_group(*g)
                .iter()
                .any(|(e, _)| *e == highlightable_e)
            {
                if state.hovered_entities.insert(highlightable_e) {
                    evw.send(HighlightEv::HoverStart(HoverData {
                        draggable: draggable.is_some(),
                        entity: highlightable_e,
                    }));
                }
            } else {
                if state.hovered_entities.remove(&highlightable_e) {
                    evw.send(HighlightEv::HoverEnd(HoverData {
                        draggable: draggable.is_some(),
                        entity: highlightable_e,
                    }));
                }
            }
        }
    }
}

fn send_drag_events(
    mut evw: EventWriter<HighlightEv>,
    mut state: ResMut<HighlightState>,
    mouse_input: Res<Input<MouseButton>>,
    added_q: Query<Entity, (With<Highligtable>, Added<Dragged>)>,
    removed: RemovedComponents<Dragged>,
) {
    for e in added_q.iter() {
        // check if not removed in the same frame
        if !removed.iter().any(|rmv_e| rmv_e == e) && state.dragged_entities.insert(e) {
            evw.send(HighlightEv::DragStart(DragData { entity: e }));
        }
    }

    // this failed when cards were applied, possibly due to stage-related reasons?
    // for e in removed.iter() {
    //     if state.dragged_entities.remove(&e) {
    //         evw.send(HighlightEv::DragEnd(DragData { entity: e }));
    //     }
    // }

    if !mouse_input.pressed(MouseButton::Left) && state.dragged_entities.len() > 0 {
        for e in state.dragged_entities.drain() {
            evw.send(HighlightEv::DragEnd(DragData { entity: e }));
        }
    }
}

fn highlight_interactable_on_drag(
    mut evr: EventReader<HighlightEv>,
    mut cmd: Commands,
    draggable_q: Query<&Draggable>,
    highlightable_q: Query<(Entity, &Highligtable)>,
    sprite_q: Query<&Sprite>,
) {
    for ev in evr.iter() {
        if let Some((e, start)) = match ev {
            HighlightEv::DragStart(data) => Some((data.entity, true)),
            HighlightEv::DragEnd(data) => Some((data.entity, false)),
            _ => None,
        } {
            for (highlightable_e, highlightable) in
                highlightable_q.iter().filter(|(e2, ..)| *e2 != e)
            {
                if let Ok(draggable) = draggable_q.get(e) {
                    tween_outline_with_drag_group_check(
                        &mut cmd,
                        highlightable,
                        highlightable_e,
                        draggable,
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
    mut evr: EventReader<HighlightEv>,
    mut cmd: Commands,
    state: Res<HighlightState>,
    sprite_q: Query<&Sprite>,
    highlightable_q: Query<(Entity, &Highligtable, Option<&Draggable>)>,
    draggable_q: Query<&Draggable>,
) {
    for ev in evr.iter() {
        if let Some((e, start)) = match ev {
            HighlightEv::HoverStart(data) => Some((data.entity, true)),
            HighlightEv::HoverEnd(data) => Some((data.entity, false)),
            _ => None,
        } {
            if let Ok((highlightable_e, highlightable, draggable)) = highlightable_q.get(e) {
                if let Some(dragged_e) = state.dragged_entities.iter().next() {
                    // already dragging => hover with a dragged item
                    if let Ok(draggable) = draggable_q.get(*dragged_e) {
                        tween_outline_with_drag_group_check(
                            &mut cmd,
                            highlightable,
                            highlightable_e,
                            draggable,
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

fn tween_outline_with_drag_group_check(
    cmd: &mut Commands,
    highlightable: &Highligtable,
    highlightable_e: Entity,
    draggable: &Draggable,
    sprite_q: &Query<&Sprite>,
    color: Color,
) {
    if highlightable
        .drag_groups
        .iter()
        .any(|g| draggable.groups.contains(&Group(*g as u8)))
    {
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
