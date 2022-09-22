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
        app.insert_resource(HighlightState::default())
            .add_system(send_hover_events)
            .add_system_to_stage(CoreStage::PostUpdate, send_drag_events)
            .add_system_to_stage(CoreStage::PostUpdate, restore_color)
            .add_system(highlight_interactable)
            .add_system(highlight_draggable_on_hover);
    }
}

#[derive(Default)]
struct HighlightState {
    dragged_entities: HashSet<Entity>,
    hovered_entities: HashSet<Entity>,
}

struct HoverData {
    draggable: bool,
}

enum HoverEv {
    HoverStart(HoverData),
    HoverEnd(HoverData),
    DragStart(Entity),
    DragEnd(Entity),
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
                    // todo: hover start ev
                    info!("hover start, draggable: {}", draggable.is_some());
                }
            } else {
                if state.hovered_entities.remove(&highlightable_e) {
                    // todo: hover start ev
                    info!("hover end, draggable: {}", draggable.is_some());
                }
            }
        }
    }
}

fn send_drag_events(
    mut state: ResMut<HighlightState>,
    added_q: Query<Entity, (With<Highligtable>, Added<Dragged>)>,
    removed: RemovedComponents<Dragged>,
    highlightable_q: Query<Entity, (With<Highligtable>, With<Draggable>)>,
) {
    for e in added_q.iter() {
        // check if not removed in the same frame
        if !removed.iter().any(|rmv_e| rmv_e == e) && state.dragged_entities.insert(e) {
            info!("drag start");
        }
    }

    for e in removed.iter().filter(|e| highlightable_q.contains(*e)) {
        if state.dragged_entities.remove(&e) {
            info!("drag end");
        }
    }
}

fn highlight_interactable(
    mut cmd: Commands,
    dragged_q: Query<(Entity, &Draggable), Added<Dragged>>,
    highlightable_q: Query<(Entity, &Highligtable)>,
    sprite_q: Query<&Sprite>,
) {
    if let Ok((dragged_e, draggable)) = dragged_q.get_single() {
        for (highlightable_e, highlightable) in
            highlightable_q.iter().filter(|(e, ..)| *e != dragged_e)
        {
            tween_outline(
                &mut cmd,
                highlightable,
                highlightable_e,
                draggable,
                &sprite_q,
                highlightable.hightlight_color,
            );
        }
    }
}

fn highlight_draggable_on_hover(
    mut cmd: Commands,
    sprite_q: Query<&Sprite>,
    mouse_input: Res<Input<MouseButton>>,
    interaction_state: Res<InteractionState>,
    highlightable_q: Query<(Entity, &Highligtable, &Draggable)>,
) {
    if !mouse_input.pressed(MouseButton::Left) || mouse_input.just_released(MouseButton::Left) {
        for (highlightable_e, highlightable, draggable) in highlightable_q.iter() {
            if let Some(g) = draggable.groups.first() {
                if interaction_state
                    .get_group(DragGroup::Card.into())
                    .iter()
                    .any(|(e, _)| *e == highlightable_e)
                {
                    tween_outline(
                        &mut cmd,
                        highlightable,
                        highlightable_e,
                        draggable,
                        &sprite_q,
                        highlightable.hover_color,
                    );
                }
            }
        }
    }
}

fn restore_color(
    mut cmd: Commands,
    removed: RemovedComponents<Dragged>,
    draggable_q: Query<&Draggable>,
    highlightable_q: Query<(Entity, &Highligtable)>,
    sprite_q: Query<&Sprite>,
) {
    for dragged_e in removed.iter() {
        if let Ok(draggable) = draggable_q.get(dragged_e) {
            for (highlightable_e, highlightable) in
                highlightable_q.iter().filter(|(e, ..)| *e != dragged_e)
            {
                tween_outline(
                    &mut cmd,
                    highlightable,
                    highlightable_e,
                    draggable,
                    &sprite_q,
                    highlightable.normal_color,
                );
            }
        }
    }
}

fn tween_outline(
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
        let e = highlightable.sprite_e.unwrap_or(highlightable_e);
        if sprite_q.contains(e) {
            cmd.entity(e)
                .insert(get_relative_sprite_color_anim(color, 220, None));
        } else {
            cmd.entity(e)
                .insert(get_relative_spritesheet_color_anim(color, 220, None));
        }
    }
}
