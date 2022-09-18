use bevy::prelude::*;
use bevy_tweening::Animator;
use iyes_loopless::prelude::AppLooplessStateExt;

use crate::{
    assets::Sprites,
    render::ZIndex,
    tile_placement::BOARD_SHIFT,
    tween::{delay_tween, get_relative_move_anim, get_relative_move_tween},
    GameState,
};

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
    let pos = Vec3::new(BOARD_SHIFT.x + 25., -1500., 0.);
    cmd.spawn_bundle(SpriteBundle {
        texture: sprites.parchment.clone(),
        transform: Transform::from_translation(pos),
        ..default()
    })
    .insert(ZIndex::Grid)
    .insert(Animator::new(delay_tween(
        get_relative_move_tween(Vec3::new(pos.x, -580., 0.), 600, None),
        1000,
    )))
    .insert(Name::new("Parchment"));
}
