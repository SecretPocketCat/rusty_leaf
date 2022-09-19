use crate::{
    anim::SheetAnimation,
    assets::{Fonts, Sprites},
    card::{CardEffect, Ingredient},
    drag::DragGroup,
    progress::TooltipProgress,
    render::{NoRescale, ZIndex, OUTLINE_COL, SCALE_MULT},
    tween::{
        get_relative_fade_text_anim, get_relative_move_anim, FadeHierarchyBundle, FadeHierarchySet,
    },
    GameState,
};
use bevy::{prelude::*, utils::HashMap};
use bevy_interact_2d::Interactable;
use iyes_loopless::prelude::*;
use std::{mem, time::Duration};

pub struct CauldronPlugin;
impl Plugin for CauldronPlugin {
    fn build(&self, app: &mut App) {
        app.add_exit_system(GameState::Loading, setup)
            .add_system(cook)
            .add_system(set_fire_intensity.after(cook))
            .add_system(show_progress_tooltip)
            .add_system(add_ingredient_to_tooltip.run_not_in_state(GameState::Loading));
    }
}

pub const COOK_TIME: f32 = 15.;
pub const FIRE_BOOST_TIME: f32 = 15.;
pub const FIRE_BOOST_MULT: f32 = 2.5;

#[derive(Component)]
pub struct Cauldron {
    pub ingredients: Vec<Ingredient>,
    pub cooked: Option<Vec<Ingredient>>,
    pub cook_timer: Timer,
    pub fire_boost: Timer,
    pub fire_e: Entity,
    pub tooltip_e: Entity,
}

pub struct TooltipIngredient {
    count: u8,
    entity: Entity,
    text_e: Entity,
}

#[derive(Component, Default)]
pub struct TooltipIngridientList {
    pub ingredients: HashMap<u8, TooltipIngredient>,
}

pub fn spawn_tooltip_ingredient(
    ingredient: Ingredient,
    count: u8,
    current_list_len: usize,
    y_offset: f32,
    cmd: &mut Commands,
    sprites: &Sprites,
    fonts: &Fonts,
) -> (Entity, Entity) {
    let x_offset = 16.;

    let x = match current_list_len {
        0 => -x_offset,
        1 => 0.,
        2 => x_offset,
        _ => panic!("Too many items"),
    };

    let txt_e = cmd
        .spawn_bundle(Text2dBundle {
            text: Text::from_section(
                format!("{count}x"),
                TextStyle {
                    font: fonts.tooltip.clone(),
                    font_size: 16.0 * SCALE_MULT,
                    color: Color::NONE,
                },
            )
            .with_alignment(TextAlignment::BOTTOM_CENTER),
            transform: Transform::from_scale(Vec2::splat(1. / SCALE_MULT).extend(1.))
                .with_translation(Vec3::new(0., 6., 0.)),
            ..default()
        })
        .insert(get_relative_fade_text_anim(
            // todo: from res/const
            OUTLINE_COL,
            400,
            None,
        ))
        .id();

    let tooltip_e = cmd
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: sprites.ingredients.clone(),
            sprite: TextureAtlasSprite {
                color: Color::NONE,
                index: ingredient.get_sprite_index(),
                ..default()
            },
            transform: Transform::from_translation(Vec2::new(x, y_offset).extend(0.01)),
            ..default()
        })
        .insert(NoRescale)
        .insert(Name::new("tooltip_ingredient"))
        .add_child(txt_e.clone())
        .id();

    (tooltip_e, txt_e)
}

fn setup(mut cmd: Commands, sprites: Res<Sprites>) {
    for (x, firepit_x, sprite_index, flip_x, fire_x) in
        [(80., -2.5, 0, false, 0.), (295., -6., 1, true, -2.0)].iter()
    {
        let fire_e = cmd
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: sprites.fire.clone(),
                sprite: TextureAtlasSprite {
                    index: *sprite_index,
                    flip_x: *flip_x,
                    ..default()
                },
                transform: Transform::from_xyz(*fire_x, -6., 0.01),
                ..default()
            })
            .insert(SheetAnimation::new(100).with_range(0..8, true))
            .insert(NoRescale)
            .insert(Name::new("Fire"))
            .id();

        let tooltip_e = cmd
            .spawn_bundle(SpriteBundle {
                texture: sprites.progress_tooltip.clone(),
                sprite: Sprite {
                    color: Color::NONE,
                    ..default()
                },
                transform: Transform::from_xyz(0., 0., 0.),
                ..default()
            })
            .insert(NoRescale)
            .insert(ZIndex::Tooltip)
            .insert(Name::new("Tooltip"))
            .insert(TooltipProgress::new())
            .insert(TooltipIngridientList::default())
            .id();

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
            fire_e: fire_e.clone(),
            tooltip_e: tooltip_e.clone(),
        })
        .insert(Name::new("Cauldron"))
        .add_child(fire_e)
        .add_child(tooltip_e)
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

fn cook(
    mut cauldron_q: Query<&mut Cauldron>,
    mut progress_q: Query<&mut TooltipProgress>,
    time: Res<Time>,
) {
    for mut c in cauldron_q.iter_mut() {
        c.fire_boost.tick(time.delta());

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
            } else if let Ok(mut p) = progress_q.get_mut(c.tooltip_e) {
                p.value = c.cook_timer.percent();
            }
        }
    }
}

fn show_progress_tooltip(
    mut cmd: Commands,
    mut card_evr: EventReader<CardEffect>,
    cauldron_q: Query<(Entity, &Cauldron)>,
) {
    let cooking_cauldrons: Vec<Entity> = card_evr
        .iter()
        .filter_map(|card_effect| {
            if let CardEffect::Ingredient { cauldron_e, .. } = card_effect {
                Some(*cauldron_e)
            } else {
                None
            }
        })
        .collect();

    for (c_e, c) in cauldron_q
        .iter()
        .filter(|(c_e, ..)| cooking_cauldrons.contains(c_e))
    {
        cmd.entity(c.tooltip_e)
            .insert_bundle(FadeHierarchyBundle::new(true, 450, OUTLINE_COL))
            .insert(get_relative_move_anim(Vec3::new(0., 28., 0.01), 550, None));

        // todo: progress out once the cooked food is used
    }
}

fn add_ingredient_to_tooltip(
    mut cmd: Commands,
    sprites: Res<Sprites>,
    fonts: Res<Fonts>,
    mut card_evr: EventReader<CardEffect>,
    cauldron_q: Query<(Entity, &Cauldron)>,
    mut tooltip_ingredient_q: Query<&mut TooltipIngridientList>,
    mut txt_q: Query<&mut Text>,
) {
    let cooking_cauldrons: Vec<_> = card_evr
        .iter()
        .filter_map(|card_effect| {
            if let CardEffect::Ingredient {
                cauldron_e,
                ingredient,
            } = card_effect
            {
                Some((*cauldron_e, *ingredient))
            } else {
                None
            }
        })
        .collect();

    if cooking_cauldrons.len() > 0 {
        for (c_e, c) in cauldron_q.iter() {
            let mut ingredient_list = tooltip_ingredient_q.get_mut(c.tooltip_e).unwrap();

            for (_, ingredient) in cooking_cauldrons.iter().filter(|(cc_e, ..)| *cc_e == c_e) {
                if let Some(tooltip_ingredient) =
                    ingredient_list.ingredients.get_mut(&(*ingredient as u8))
                {
                    tooltip_ingredient.count += 1;
                    let mut txt = txt_q.get_mut(tooltip_ingredient.text_e).unwrap();
                    txt.sections[0].value = format!("{}x", tooltip_ingredient.count);
                } else {
                    let (ingridient_e, ingridient_txt_e) = spawn_tooltip_ingredient(
                        *ingredient,
                        1,
                        ingredient_list.ingredients.len(),
                        -4.5,
                        &mut cmd,
                        &sprites,
                        &fonts,
                    );

                    cmd.entity(c.tooltip_e).add_child(ingridient_e.clone());

                    ingredient_list.ingredients.insert(
                        *ingredient as u8,
                        TooltipIngredient {
                            count: 1,
                            entity: ingridient_e,
                            text_e: ingridient_txt_e,
                        },
                    );
                }
            }
        }
    }
}

fn set_fire_intensity(
    cauldron_q: Query<(Entity, &Cauldron)>,
    mut fire_anim_q: Query<&mut SheetAnimation>,
    mut card_evr: EventReader<CardEffect>,
) {
    let boosted_cauldrons: Vec<_> = card_evr
        .iter()
        .filter_map(|card_effect| {
            if let CardEffect::FireBoost(cauldron_e) = card_effect {
                Some(*cauldron_e)
            } else {
                None
            }
        })
        .collect();

    for (c_e, c) in cauldron_q.iter() {
        if let Some((range, anim_dur)) = if boosted_cauldrons.contains(&c_e) {
            Some((8..16, 80))
        } else if c.fire_boost.just_finished() {
            Some((0..8, 100))
        } else {
            None
        } {
            if let Ok(mut anim) = fire_anim_q.get_mut(c.fire_e) {
                anim.set_range(range);
                anim.set_time(anim_dur);
            }
        }
    }
}
