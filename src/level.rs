use bevy::prelude::*;
use iyes_loopless::prelude::AppLooplessStateExt;

use crate::{assets::Sprites, tile_placement::BOARD_SHIFT, GameState};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_exit_system(GameState::Loading, setup_app)
            .add_enter_system(GameState::Playing, setup);
    }
}

fn setup_app(mut cmd: Commands, sprites: Res<Sprites>) {
    cmd.spawn_bundle(SpriteBundle {
        texture: sprites.bg.clone(),
        transform: Transform::from_scale(Vec2::splat(4.).extend(1.0)),
        ..default()
    })
    .insert(Name::new("bg"));
}

fn setup(mut cmd: Commands, sprites: Res<Sprites>) {
    cmd.spawn_bundle(SpriteBundle {
        texture: sprites.parchment.clone(),
        transform: Transform {
            translation: Vec3::new(BOARD_SHIFT.x + 25., -580., 0.1),
            scale: Vec2::splat(4.).extend(1.),
            ..default()
        },
        ..default()
    })
    .insert(Name::new("Parchment"));
}
