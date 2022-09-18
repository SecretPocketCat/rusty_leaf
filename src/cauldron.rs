use std::{mem, time::Duration};

use bevy::prelude::*;

use bevy_interact_2d::Interactable;
use iyes_loopless::prelude::AppLooplessStateExt;

use crate::{
    assets::Sprites,
    card::Ingredient,
    drag::DragGroup,
    render::{NoRescale, ZIndex},
    GameState,
};

pub struct CauldronPlugin;
impl Plugin for CauldronPlugin {
    fn build(&self, app: &mut App) {
        app.add_exit_system(GameState::Loading, setup)
            .add_system(cook);
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

fn setup(mut cmd: Commands, sprites: Res<Sprites>) {
    for (x, firepit_x, sprite_index) in [(65., -2.5, 0), (235., -6., 1)].iter() {
        // todo: sprites (cauldrons, wood, fire)
        cmd.spawn_bundle(SpriteSheetBundle {
            texture_atlas: sprites.cauldron.clone(),
            sprite: TextureAtlasSprite::new(*sprite_index),
            transform: Transform::from_xyz(*x, -176., 0.5),
            ..default()
        })
        .insert(ZIndex::Cauldron)
        .insert(Cauldron {
            ingredients: Vec::with_capacity(10),
            cook_timer: Timer::new(Duration::from_secs_f32(COOK_TIME), true),
            fire_boost: Timer::default(),
            cooked: None,
        })
        .insert(Name::new("Cauldron"))
        .with_children(|b| {
            b.spawn_bundle(SpriteSheetBundle {
                texture_atlas: sprites.firepit.clone(),
                sprite: TextureAtlasSprite::new(*sprite_index),
                transform: Transform::from_xyz(*firepit_x, -25., -0.01),
                ..default()
            })
            .insert(NoRescale);

            for (y, corner_x, corner_y, group) in [
                (10., 18., 18., DragGroup::Cauldron),
                (-28., 18., 16., DragGroup::Fire),
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
        if !c.ingredients.is_empty() {
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
