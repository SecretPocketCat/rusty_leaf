use bevy::prelude::*;
use bevy_tweening::Animator;
use iyes_loopless::prelude::AppLooplessStateExt;

use crate::{
    anim::SheetAnimation,
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
    for (handle, z_index, name) in [
        (sprites.bg.clone(), ZIndex::Bg, "bg"),
        (sprites.bg_shop.clone(), ZIndex::BgShop, "bg_shop"),
    ]
    .into_iter()
    {
        cmd.spawn_bundle(SpriteBundle {
            texture: handle,
            ..default()
        })
        .insert(z_index)
        .insert(Name::new(name));
    }

    for (handle, z_index, x, y, name) in [
        (
            sprites.ferris.clone(),
            ZIndex::Shopkeep,
            -350.,
            -218.,
            "ferris",
        ),
        (
            sprites.shop_smoke.clone(),
            ZIndex::BgShop,
            -470.,
            169.,
            "shop_smoke",
        ),
    ]
    .into_iter()
    {
        cmd.spawn_bundle(SpriteSheetBundle {
            texture_atlas: handle,
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_xyz(x, y, 0.0),
            ..default()
        })
        .insert(ZIndex::Shopkeep)
        .insert(SheetAnimation::new(100))
        .insert(Name::new(name));
    }
}

fn setup(mut cmd: Commands, sprites: Res<Sprites>) {
    // let pos = Vec3::new(BOARD_SHIFT.x + 25., -1500., 0.);
    // cmd.spawn_bundle(SpriteBundle {
    //     texture: sprites.parchment.clone(),
    //     transform: Transform::from_translation(pos),
    //     ..default()
    // })
    // .insert(ZIndex::Grid)
    // .insert(Animator::new(delay_tween(
    //     get_relative_move_tween(Vec3::new(pos.x, -580., 0.), 600, None),
    //     1000,
    // )))
    // .insert(Name::new("Parchment"));
}
