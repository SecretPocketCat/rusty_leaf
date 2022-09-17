use bevy::prelude::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

fn setup(mut cmd: Commands, ass: Res<AssetServer>) {
    cmd.spawn_bundle(SpriteBundle {
        texture: ass.load("sprites/bg.png"),
        transform: Transform::from_scale(Vec2::splat(4.).extend(0.)),
        ..default()
    });
}
