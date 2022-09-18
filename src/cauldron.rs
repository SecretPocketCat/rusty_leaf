use std::{collections::VecDeque, mem, time::Duration};

use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_interact_2d::{
    drag::{Draggable, Dragged, DropStrategy},
    Group, Interactable,
};

use crate::{
    board::BoardClear,
    card::Ingredient,
    drag::DragGroup,
    render::WINDOW_SIZE,
    tile_placement::{BOARD_SHIFT, SECTION_SIZE},
};

pub struct CauldronPlugin;
impl Plugin for CauldronPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system(cook);
    }
}

pub const COOK_TIME: f32 = 15.;
pub const FIRE_BOOST_TIME: f32 = 30.;
pub const FIRE_BOOST_MULT: f32 = 2.;

#[derive(Component)]
pub struct Cauldron {
    pub ingredients: Vec<Ingredient>,
    pub cooked: Option<Vec<Ingredient>>,
    pub cook_timer: Timer,
    pub fire_boost: Timer,
}

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
        .insert(Cauldron {
            ingredients: Vec::with_capacity(10),
            cook_timer: Timer::new(Duration::from_secs_f32(COOK_TIME), true),
            fire_boost: Timer::default(),
            cooked: None,
        })
        .insert(Name::new("Cauldron"))
        .with_children(|b| {
            for (y, corner_x, corner_y, group) in [
                (40., 80., 70., DragGroup::Cauldron),
                (-90., 80., 40., DragGroup::Fire),
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

fn cook(mut cauldron_q: Query<&mut Cauldron>, time: Res<Time>) {
    for mut c in cauldron_q.iter_mut() {
        // there's smt. to cook
        if c.ingredients.len() > 0 {
            let mult = if c.fire_boost.finished() {
                1.
            } else {
                FIRE_BOOST_MULT
            };
            c.cook_timer.tick(time.delta().mul_f32(mult));

            if c.cook_timer.just_finished() {
                c.cooked = Some(mem::take(&mut c.ingredients));
                info!("Soup's done!");
            }
        }
    }
}
