use bevy::prelude::*;
use bevy_interact_2d::InteractionSource;

pub struct RenderPlugin;

#[derive(Component)]
pub struct MainCam;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

fn setup(mut cmd: Commands) {
    info!("render setup");
    cmd.spawn_bundle(Camera2dBundle::default())
        .insert(MainCam)
        .insert(InteractionSource {
            groups: vec![bevy_interact_2d::Group(0)],
            ..Default::default()
        });
}
