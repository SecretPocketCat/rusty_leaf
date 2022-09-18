use bevy::prelude::*;
use iyes_loopless::prelude::AppLooplessStateExt;

use crate::{assets::Sprites, render::ZIndex, tile_placement::BOARD_SHIFT, GameState};

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
        ..default()
    })
    .insert(ZIndex::Bg)
    .insert(Name::new("bg"));
}

fn setup(mut cmd: Commands, sprites: Res<Sprites>) {
    cmd.spawn_bundle(SpriteBundle {
        texture: sprites.parchment.clone(),
        transform: Transform::from_xyz(BOARD_SHIFT.x + 25., -580., 0.),
        ..default()
    })
    .insert(ZIndex::Grid)
    .insert(Name::new("Parchment"));
}
