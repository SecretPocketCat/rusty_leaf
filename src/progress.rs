use crate::{assets::Sprites, GameState};
use bevy::prelude::*;

use iyes_loopless::prelude::*;

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
    offset: f32,
    inverse: bool,
}

impl TooltipProgress {
    pub fn new(offset: f32, inverse: bool) -> Self {
        Self {
            offset,
            inverse,
            ..default()
        }
    }

    pub fn progress(&self) -> f32 {
        if self.inverse {
            1. - self.value
        } else {
            self.value
        }
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
                texture: if progress.inverse {
                    sprites.progress_bar_order.clone()
                } else {
                    sprites.progress_bar.clone()
                },
                transform: Transform::from_xyz(-6., 4. + progress.offset, 0.1)
                    .with_scale(Vec3::new(progress.progress(), 1., 1.)),
                sprite: Sprite {
                    anchor: bevy::sprite::Anchor::CenterLeft,
                    ..default()
                },
                ..default()
            })
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
                t.scale.x = p.progress();
            }
        }
    }
}
