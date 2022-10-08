use crate::{
    anim::SheetAnimation,
    assets::{Fonts, Sprites},
    card::{CardEffect, Ingredient},
    highlight::Highligtable,
    interaction::{Interactable, InteractionGroup},
    level::LevelEv,
    order::{Order, OrderEv},
    progress::TooltipProgress,
    render::{
        ZIndex, COL_DARK, COL_DARKER, COL_OUTLINE_HIGHLIGHTED, COL_OUTLINE_HIGHLIGHTED_2,
        COL_OUTLINE_HOVERED_DRAG,
    },
    tween::{
        get_relative_fade_text_anim, get_relative_move_anim, get_relative_move_by_anim,
        get_relative_spritesheet_color_anim, FadeHierarchy, FadeHierarchyBundle, TweenDoneAction,
    },
    GameState,
};
use bevy::{prelude::*, utils::HashMap};
use iyes_loopless::prelude::*;
use std::time::Duration;

pub struct CauldronPlugin;
impl Plugin for CauldronPlugin {
    fn build(&self, app: &mut App) {
        app.add_exit_system(GameState::Loading, setup)
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(GameState::Loading)
                    .with_system(show_progress_tooltip)
                    .with_system(add_ingredient_to_tooltip)
                    .with_system(boost_fire)
                    .with_system(on_level_over)
                    .into(),
            )
            .add_system(cook)
            .add_system(set_fire_intensity.after(cook));
    }
}

// pub const COOK_TIME: f32 = 1.;
pub const COOK_TIME: f32 = 15.;
pub const FIRE_BOOST_TIME: f32 = 15.;
pub const FIRE_BOOST_MULT: f32 = 2.5;
const TOOLTIP_TWEEN_OFFSET: f32 = 28.;
const CAULDRON_INGREDIENT_Y: f32 = -4.5;

#[derive(Component)]
pub struct Cauldron {
    pub ingredients: Vec<Ingredient>,
    pub cook_timer: Timer,
    pub fire_boost: Timer,
    pub fire_e: Entity,
    pub tooltip_e: Option<Entity>,
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
                    font_size: 16.0,
                    color: Color::NONE,
                },
            )
            .with_alignment(TextAlignment::BOTTOM_CENTER),
            transform: Transform::from_xyz(0., 6., 0.),
            ..default()
        })
        .insert(get_relative_fade_text_anim(
            // todo: from res/const
            COL_DARK, 400, None,
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
        .insert(get_relative_spritesheet_color_anim(Color::WHITE, 250, None))
        .insert(Name::new("tooltip_ingredient"))
        .add_child(txt_e)
        .id();

    (tooltip_e, txt_e)
}

fn setup(mut cmd: Commands, sprites: Res<Sprites>) {
    for (x, firepit_x, sprite_index, flip_x, fire_x) in
        [(20., -1., 0, false, 0.), (74., -5., 1, true, -1.0)].iter()
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
            .insert(Name::new("Fire"))
            .id();

        let cauldron_outline_e = cmd
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: sprites.cauldron_outline.clone(),
                sprite: TextureAtlasSprite {
                    index: *sprite_index,
                    color: COL_DARKER,
                    ..default()
                },
                ..default()
            })
            .insert(Name::new("outline"))
            .id();

        let firepit_outline_e = cmd
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: sprites.firepit_outline.clone(),
                sprite: TextureAtlasSprite {
                    index: *sprite_index,
                    color: COL_DARKER,
                    ..default()
                },
                ..default()
            })
            .insert(Name::new("outline"))
            .id();

        cmd.spawn_bundle(SpriteSheetBundle {
            texture_atlas: sprites.cauldron.clone(),
            sprite: TextureAtlasSprite::new(*sprite_index),
            transform: Transform::from_xyz(*x, -44., 0.5),
            ..default()
        })
        .insert(ZIndex::Cauldron)
        .insert(Cauldron {
            ingredients: Vec::with_capacity(10),
            cook_timer: Timer::new(Duration::from_secs_f32(COOK_TIME), true),
            fire_boost: Timer::default(),
            fire_e,
            tooltip_e: None,
        })
        .insert(Name::new("Cauldron"))
        .add_child(fire_e)
        .add_child(cauldron_outline_e)
        .with_children(|b| {
            b.spawn_bundle(SpriteSheetBundle {
                texture_atlas: sprites.firepit.clone(),
                sprite: TextureAtlasSprite::new(*sprite_index),
                transform: Transform::from_xyz(*firepit_x, -25., -0.01),
                ..default()
            })
            .add_child(firepit_outline_e);

            for (y, corner_x, corner_y, group, outline_e, highlight_col) in [
                (
                    10.,
                    18.,
                    18.,
                    InteractionGroup::Cauldron,
                    cauldron_outline_e,
                    COL_OUTLINE_HIGHLIGHTED,
                ),
                (
                    -28.,
                    18.,
                    16.,
                    InteractionGroup::Fire,
                    firepit_outline_e,
                    COL_OUTLINE_HIGHLIGHTED_2,
                ),
            ] {
                let corner = Vec2::new(corner_x, corner_y);
                b.spawn_bundle(SpatialBundle {
                    transform: Transform::from_xyz(0., y, 0.),
                    ..default()
                })
                .insert(Interactable::new_rectangle(group, corner))
                .insert(Highligtable {
                    sprite_e: Some(outline_e),
                    hightlight_color: highlight_col,
                    hover_color: COL_OUTLINE_HOVERED_DRAG,
                    normal_color: COL_DARK,
                    drag_groups: vec![InteractionGroup::Card],
                });
            }
        });
    }
}

fn cook(
    mut cmd: Commands,
    mut cauldron_q: Query<(Entity, &mut Cauldron)>,
    mut progress_q: Query<&mut TooltipProgress>,
    order_q: Query<(Entity, &Order)>,
    time: Res<Time>,
    mut order_evw: EventWriter<OrderEv>,
    mut card_evw: EventWriter<CardEffect>,
) {
    for (c_e, mut c) in cauldron_q.iter_mut() {
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
                if let Some((order_e, _)) = order_q
                    .iter()
                    .find(|(_order_e, o)| o.is_equal(&c.ingredients))
                {
                    // complete order
                    order_evw.send(OrderEv::Completed(order_e));
                } else {
                    // burn the invalid soup
                    card_evw.send(CardEffect::FireBoost {
                        cauldron_e: c_e,
                        boost_dur_multiplier: Some(2.),
                    });
                }

                clear_cauldron_ingredients(&mut cmd, &mut c);
            } else if let Some(tooltip_e) = c.tooltip_e {
                if let Ok(mut p) = progress_q.get_mut(tooltip_e) {
                    p.value = c.cook_timer.percent();
                }
            }
        }
    }
}

fn clear_cauldron_ingredients(cmd: &mut Commands, cauldron: &mut Cauldron) {
    if let Some(tooltip_e) = cauldron.tooltip_e {
        let mut tooltip_cmd_e = cmd.entity(tooltip_e);
        tooltip_cmd_e.insert(FadeHierarchy::new(false, 350, Color::NONE));
        tooltip_cmd_e.insert(get_relative_move_by_anim(
            Vec3::Y * -TOOLTIP_TWEEN_OFFSET,
            400,
            Some(TweenDoneAction::DespawnRecursive),
        ));
        cauldron.tooltip_e = None;
        cauldron.ingredients.clear();
    }
}

fn show_progress_tooltip(
    mut cmd: Commands,
    mut card_evr: EventReader<CardEffect>,
    mut cauldron_q: Query<&mut Cauldron>,
    sprites: Res<Sprites>,
    fonts: Res<Fonts>,
) {
    for ev in card_evr.iter() {
        if let CardEffect::Ingredient {
            ingredient,
            cauldron_e,
        } = ev
        {
            if let Ok(mut c) = cauldron_q.get_mut(*cauldron_e) {
                if c.tooltip_e.is_none() {
                    let (ingredient_e, ingridient_txt_e) = spawn_tooltip_ingredient(
                        *ingredient,
                        1,
                        0,
                        CAULDRON_INGREDIENT_Y,
                        &mut cmd,
                        &sprites,
                        &fonts,
                    );

                    cmd.entity(*cauldron_e).with_children(|b| {
                        let mut ingredient_list = TooltipIngridientList::default();
                        ingredient_list.ingredients.insert(
                            *ingredient as u8,
                            TooltipIngredient {
                                count: 1,
                                entity: ingredient_e,
                                text_e: ingridient_txt_e,
                            },
                        );

                        c.tooltip_e = Some(
                            b.spawn_bundle(SpriteBundle {
                                texture: sprites.progress_tooltip.clone(),
                                sprite: Sprite {
                                    color: Color::NONE,
                                    ..default()
                                },
                                ..default()
                            })
                            .insert(ZIndex::Tooltip)
                            .insert(Name::new("Tooltip"))
                            .insert(TooltipProgress::new(0., false))
                            .insert(ingredient_list)
                            .insert_bundle(FadeHierarchyBundle::new(true, 450, COL_DARK))
                            .insert(get_relative_move_anim(
                                Vec3::new(0., TOOLTIP_TWEEN_OFFSET, 0.01),
                                550,
                                None,
                            ))
                            .add_child(ingredient_e)
                            .id(),
                        );
                    });
                }
            }
        }
    }
}

fn add_ingredient_to_tooltip(
    mut cmd: Commands,
    sprites: Res<Sprites>,
    fonts: Res<Fonts>,
    mut card_evr: EventReader<CardEffect>,
    cauldron_q: Query<&Cauldron>,
    mut tooltip_ingredient_q: Query<&mut TooltipIngridientList>,
    mut txt_q: Query<&mut Text>,
) {
    for ev in card_evr.iter() {
        if let CardEffect::Ingredient {
            ingredient,
            cauldron_e,
        } = ev
        {
            if let Ok(c) = cauldron_q.get(*cauldron_e) {
                if let Some(tooltip_e) = c.tooltip_e {
                    if let Ok(mut ingredient_list) = tooltip_ingredient_q.get_mut(tooltip_e) {
                        if let Some(tooltip_ingredient) =
                            ingredient_list.ingredients.get_mut(&(*ingredient as u8))
                        {
                            tooltip_ingredient.count += 1;
                            let mut txt = txt_q.get_mut(tooltip_ingredient.text_e).unwrap();
                            txt.sections[0].value = format!("{}x", tooltip_ingredient.count);
                        } else {
                            let (ingredient_e, ingredient_txt_e) = spawn_tooltip_ingredient(
                                *ingredient,
                                1,
                                ingredient_list.ingredients.len(),
                                CAULDRON_INGREDIENT_Y,
                                &mut cmd,
                                &sprites,
                                &fonts,
                            );

                            cmd.entity(c.tooltip_e.unwrap()).add_child(ingredient_e);

                            ingredient_list.ingredients.insert(
                                *ingredient as u8,
                                TooltipIngredient {
                                    count: 1,
                                    entity: ingredient_e,
                                    text_e: ingredient_txt_e,
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

fn boost_fire(mut cauldron_q: Query<&mut Cauldron>, mut card_evr: EventReader<CardEffect>) {
    for ev in card_evr.iter() {
        if let CardEffect::FireBoost {
            cauldron_e,
            boost_dur_multiplier,
        } = ev
        {
            if let Ok(mut c) = cauldron_q.get_mut(*cauldron_e) {
                let dur = c
                    .fire_boost
                    .duration()
                    .saturating_add(Duration::from_secs_f32(
                        FIRE_BOOST_TIME * boost_dur_multiplier.unwrap_or(1.),
                    ));
                c.fire_boost.set_duration(dur);
                c.fire_boost.reset();
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
            if let CardEffect::FireBoost { cauldron_e, .. } = card_effect {
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

fn on_level_over(
    mut cmd: Commands,
    mut lvl_evr: EventReader<LevelEv>,
    mut cauldron_q: Query<&mut Cauldron>,
) {
    for ev in lvl_evr.iter() {
        if let LevelEv::LevelOver { .. } = ev {
            // hide tooltips
            for mut c in cauldron_q.iter_mut() {
                // end fire boosts
                c.fire_boost.reset();
                c.fire_boost.set_duration(Duration::from_secs(1));
                clear_cauldron_ingredients(&mut cmd, &mut c);
            }

            break;
        }
    }
}
