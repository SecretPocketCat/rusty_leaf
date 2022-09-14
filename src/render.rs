use bevy::prelude::*;

pub struct RenderPlugin;

#[derive(Component)]
pub struct MainCam;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

fn setup(mut cmd: Commands) {
    cmd.spawn_bundle(Camera2dBundle::default()).insert(MainCam);
}
