use crate::{
    anim::SheetAnimation,
    assets::Sprites,
    card::{CardEffect, Ingredient},
    drag::DragGroup,
    render::{NoRescale, ZIndex},
    GameState,
};
use bevy::prelude::*;
use bevy_interact_2d::Interactable;
use image::Progress;
use iyes_loopless::prelude::*;
use std::{mem, time::Duration};

pub struct ProgressPlugin;
impl Plugin for ProgressPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(on_progress_added.run_not_in_state(GameState::Loading))
            .add_system(update_progress);
    }
}

#[derive(Component, Default)]
pub struct TooltipProgress {
    pub value: f32,
    progress_sprite_e: Option<Entity>,
}

impl TooltipProgress {
    pub fn new() -> Self {
        Self::default()
    }
}

fn on_progress_added(
    mut cmd: Commands,
    sprites: Res<Sprites>,
    mut progress_q: Query<(Entity, &mut TooltipProgress), Added<TooltipProgress>>,
) {
    for (progress_e, mut progress) in progress_q.iter_mut() {
        let bar_e = cmd
            .spawn_bundle(SpriteBundle {
                texture: sprites.progress_bar.clone(),
                transform: Transform::from_xyz(-24.6, 14., 0.1).with_scale(Vec3::new(
                    progress.value,
                    1.,
                    1.,
                )),
                sprite: Sprite {
                    anchor: bevy::sprite::Anchor::CenterLeft,
                    ..default()
                },
                ..default()
            })
            .insert(NoRescale)
            .id();

        progress.progress_sprite_e = Some(bar_e);
        cmd.entity(progress_e).add_child(bar_e);
    }
}

fn update_progress(
    progress_q: Query<&TooltipProgress, Changed<TooltipProgress>>,
    mut transform_q: Query<&mut Transform>,
) {
    for p in progress_q.iter() {
        if let Some(e) = p.progress_sprite_e {
            if let Ok(mut t) = transform_q.get_mut(e) {
                t.scale.x = p.value;
            }
        }
    }
}
