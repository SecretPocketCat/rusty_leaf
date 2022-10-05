use crate::{
    drag::{Draggable, Dragged},
    mouse::CursorWorldPosition,
};
use bevy::{
    prelude::*,
    sprite::Rect,
    utils::{HashMap, HashSet},
};
use bevy_tweening::Animator;
use strum::{EnumIter, IntoEnumIterator};

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractionEv>()
            .insert_resource(InteractionState::default())
            .add_system(check_interaction);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum InteractionGroup {
    Piece = 1,
    Card,
    Cauldron,
    Fire,
    Grid,
    GridPieces,
    GridSection,
}

#[derive(Debug, Component)]
pub struct Interactable {
    pub group: InteractionGroup,
    pub bounds: Rect,
    pub enabled: bool,
}

impl Interactable {
    pub fn new(group: InteractionGroup, min: Vec2, max: Vec2) -> Self {
        Self {
            group,
            bounds: Rect { min, max },
            enabled: true,
        }
    }

    pub fn new_rectangle(group: InteractionGroup, max: Vec2) -> Self {
        Self::new(group, -max, max)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        self.bounds.min.x <= point.x
            && self.bounds.max.x >= point.x
            && self.bounds.min.y <= point.y
            && self.bounds.max.y >= point.y
    }
}

pub struct InteractionState {
    pub dragged_e: Option<Entity>,
    hovered_entities: HashMap<InteractionGroup, HashSet<Entity>>,
}

impl Default for InteractionState {
    fn default() -> Self {
        let hovered = InteractionGroup::iter()
            .map(|g| (g, HashSet::new()))
            .collect();

        Self {
            dragged_e: None,
            hovered_entities: hovered,
        }
    }
}

impl InteractionState {
    pub fn get_hovered_entities(&self, group: &InteractionGroup) -> Vec<Entity> {
        self.hovered_entities
            .get(group)
            .map_or_else(|| Vec::new(), |hovered| hovered.iter().cloned().collect())
    }

    pub fn get_first_hovered_entity(&self, group: &InteractionGroup) -> Option<Entity> {
        self.hovered_entities
            .get(group)
            .unwrap()
            .iter()
            .next()
            .cloned()
    }
}

pub struct HoverData {
    pub e: Entity,
    pub draggable: bool,
}

pub struct DragData {
    pub e: Entity,
    pub origin: Vec2,
}

pub enum InteractionEv {
    HoverStart(HoverData),
    HoverEnd(HoverData),
    DragStart(DragData),
    DragEnd(DragData),
}

fn check_interaction(
    mut cmd: Commands,
    mut state: ResMut<InteractionState>,
    cursor: Res<CursorWorldPosition>,
    mouse_input: Res<Input<MouseButton>>,
    mut evw: EventWriter<InteractionEv>,
    interactable_q: Query<(
        Entity,
        &Interactable,
        &GlobalTransform,
        ChangeTrackers<GlobalTransform>,
        Option<&Draggable>,
        Option<&Animator<Transform>>,
    )>,
    dragged_q: Query<&Dragged>,
) {
    if let Some(e) = state.dragged_e && !mouse_input.pressed(MouseButton::Left) {
        state.dragged_e = None;
        evw.send(InteractionEv::DragEnd(DragData { e, origin: dragged_q.get(e).map_or(Vec2::ZERO, |dragged| dragged.origin) }));
    }

    for (e, i, t, t_change, draggable, tween) in interactable_q.iter() {
        if cursor.is_changed() || t_change.is_changed() {
            let pos = t.translation().truncate();

            if i.contains(cursor.0 - pos) {
                if state.hovered_entities.get_mut(&i.group).unwrap().insert(e) {
                    evw.send(InteractionEv::HoverStart(HoverData {
                        draggable: draggable.is_some(),
                        e,
                    }));
                }

                if state.dragged_e.is_none()
                    && mouse_input.just_pressed(MouseButton::Left)
                    && draggable.is_some()
                {
                    // no running transform tween
                    if tween.map_or(true, |t| t.tweenable().progress() >= 1.) {
                        state.dragged_e = Some(e);
                        evw.send(InteractionEv::DragStart(DragData { e, origin: pos }));
                    }
                }
            } else {
                if state.hovered_entities.get_mut(&i.group).unwrap().remove(&e) {
                    evw.send(InteractionEv::HoverEnd(HoverData {
                        draggable: draggable.is_some(),
                        e,
                    }));
                }
            }
        }
    }
}
