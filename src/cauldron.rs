use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_interact_2d::{
    drag::{Draggable, Dragged, DropStrategy},
    Group, Interactable,
};

use crate::{
    board::BoardClear,
    drag::DragGroup,
    render::WINDOW_SIZE,
    tile_placement::{BOARD_SHIFT, SECTION_SIZE},
};

pub struct CauldronPlugin;
impl Plugin for CauldronPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

#[derive(Component)]
pub struct Cauldron;

fn setup(mut cmd: Commands) {
    for x in [60., 230.].iter() {
        // todo: sprites (cauldrons, wood, fire)
        cmd.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(150.)),
                color: Color::CYAN,
                ..default()
            },
            transform: Transform::from_xyz(*x, -205., 0.5),
            ..default()
        })
        .insert(Name::new("Cauldron"))
        .with_children(|b| {
            for (y, corner_x, corner_y, group) in [
                (40., 80., 70., DragGroup::Cauldron),
                (-90., 80., 40., DragGroup::CauldronFire),
            ] {
                let corner = Vec2::new(corner_x, corner_y);
                b.spawn_bundle(SpatialBundle {
                    transform: Transform::from_xyz(0., y, 0.),
                    ..default()
                })
                .insert(Interactable {
                    bounding_box: (-corner, corner),
                    groups: vec![group.into()],
                });
            }
        });
    }
}
