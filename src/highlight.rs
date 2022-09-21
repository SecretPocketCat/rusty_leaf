use bevy::prelude::*;
use bevy_interact_2d::{
    drag::{Draggable, Dragged},
    Group, Interactable,
};
use bevy_tweening::*;

use crate::{
    drag::DragGroup,
    render::COL_DARK,
    tween::{get_relative_sprite_color_anim, get_relative_spritesheet_color_anim},
};

pub struct HighlightPlugin;
impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app // .add_system_to_stage(CoreStage::PostUpdate, disable_drag_during_tween)
            .add_system(highlight_interactable);
    }
}

#[derive(Component)]
pub struct Highligtable {
    pub sprite_e: Option<Entity>,
    pub hightlight_color: Color,
    pub normal_color: Color,
    pub drag_groups: Vec<DragGroup>,
}

// todo: reset back
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
            if highlightable
                .drag_groups
                .iter()
                .any(|g| draggable.groups.contains(&Group(*g as u8)))
            {
                let e = highlightable.sprite_e.unwrap_or(highlightable_e);
                if sprite_q.contains(e) {
                    cmd.entity(e).insert(get_relative_sprite_color_anim(
                        highlightable.hightlight_color,
                        150,
                        None,
                    ));
                } else {
                    cmd.entity(e).insert(get_relative_spritesheet_color_anim(
                        highlightable.hightlight_color,
                        220,
                        None,
                    ));
                }
            }
        }
    }
}

fn highlight_draggable() {}
