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
    cmd.spawn_bundle(Camera2dBundle::default())
        // replace default cam pos to shift the tilegrid so that it's still centered in the world 'cause I can't be bothered to fix the coordinates for shifting it off-center
        .insert(Transform::from_xyz(260., 0., 999.))
        .insert(MainCam)
        .insert(InteractionSource {
            groups: vec![bevy_interact_2d::Group(0)],
            ..Default::default()
        });
}
