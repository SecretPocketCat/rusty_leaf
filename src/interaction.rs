use crate::mouse::CursorWorldPosition;
use bevy::{
    prelude::*,
    sprite::Rect,
    utils::{HashMap, HashSet},
};

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractionEv>()
            .insert_resource(InteractionState::default())
            .add_system(check_interaction)
            // .add_system(send_hover_events)
            // .add_system(send_drag_events)
            ;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Component)]
pub struct Draggable;

#[derive(Component)]
pub struct Dragged {
    pub origin: Vec2,
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

#[derive(Default)]
pub struct InteractionState {
    pub dragged_e: Option<Entity>,
    pub hovered_entities: HashMap<InteractionGroup, HashSet<Entity>>,
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
    cursor: Res<CursorWorldPosition>,
    interactable_q: Query<(
        &Interactable,
        &GlobalTransform,
        ChangeTrackers<GlobalTransform>,
    )>,
) {
    for (i, t, t_change) in interactable_q.iter() {
        if cursor.is_changed() || t_change.is_changed() {
            let pos = t.translation().truncate();

            if i.contains(cursor.0 - pos) {
                info!("intersection: {pos}");
            }
        }
    }
}

// fn send_hover_events(
//     mut evw: EventWriter<InteractionEv>,
//     mut state: ResMut<InteractionState>,
//     interaction_state: Res<InteractionState>,
//     interactable_q: Query<(Entity, &Interactable, Option<&Draggable>)>,
// ) {
//     for (e, interactable, draggable) in interactable_q.iter() {
//         if interaction_state.get_group(*g).iter().any(|(e, _)| *e == e) {
//             if state.hovered_entities.insert(e) {
//                 evw.send(InteractionEv::HoverStart(HoverData {
//                     draggable: draggable.is_some(),
//                     entity: e,
//                 }));
//             }
//         } else {
//             if state.hovered_entities.remove(&e) {
//                 evw.send(InteractionEv::HoverEnd(HoverData {
//                     draggable: draggable.is_some(),
//                     entity: e,
//                 }));
//             }
//         }
//     }
// }

// fn send_drag_events(
//     mut evw: EventWriter<InteractionEv>,
//     mut state: ResMut<InteractionState>,
//     mouse_input: Res<Input<MouseButton>>,
//     added_q: Query<Entity, Added<Dragged>>,
//     removed: RemovedComponents<Dragged>,
// ) {
//     for e in added_q.iter() {
//         // check if not removed in the same frame
//         if !removed.iter().any(|rmv_e| rmv_e == e) && state.dragged_e.insert(e) {
//             evw.send(InteractionEv::DragStart(DragData { entity: e }));
//         }
//     }

//     for e in removed.iter() {
//         if state.dragged_e.remove(&e) {
//             evw.send(InteractionEv::DragEnd(DragData { entity: e }));
//         }
//     }

//     if !mouse_input.pressed(MouseButton::Left) && state.dragged_e.len() > 0 {
//         for e in state.dragged_e.drain() {
//             evw.send(InteractionEv::DragEnd(DragData { entity: e }));
//         }
//     }
// }
