use bevy::prelude::*;

use crate::tile_placement::BOARD_SHIFT;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

fn setup(mut cmd: Commands, ass: Res<AssetServer>) {
    cmd.spawn_bundle(SpriteBundle {
        texture: ass.load("sprites/bg.png"),
        transform: Transform::from_scale(Vec2::splat(4.).extend(1.0)),
        ..default()
    });

    cmd.spawn_bundle(SpriteBundle {
        texture: ass.load("sprites/parchment.png"),
        transform: Transform {
            translation: Vec3::new(BOARD_SHIFT.x + 25., -580., 0.1),
            scale: Vec2::splat(4.).extend(1.),
            ..default()
        },
        ..default()
    })
    .insert(Name::new("Parchment"));
}
