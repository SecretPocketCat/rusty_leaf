use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_interact_2d::InteractionSource;

use crate::drag::DragGroup;

pub struct RenderPlugin;

#[derive(Component)]
pub struct MainCam;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

pub const WINDOW_SIZE: Vec2 = Vec2::new(1280., 720.);

fn setup(mut cmd: Commands) {
    cmd.spawn_bundle(Camera2dBundle::default())
        // replace default cam pos to shift the tilegrid so that it's still centered in the world 'cause I can't be bothered to fix the coordinates for shifting it off-center
        // .insert(Transform::from_xyz(260., 0., 999.))
        .insert(MainCam)
        .insert(InteractionSource {
            groups: vec![
                DragGroup::Card.into(),
                DragGroup::Piece.into(),
                DragGroup::Cauldron.into(),
                DragGroup::Fire.into(),
                DragGroup::Grid.into(),
                DragGroup::GridPieces.into(),
            ],
            ..Default::default()
        });
}
