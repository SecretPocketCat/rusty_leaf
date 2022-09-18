use crate::{
    assets::Sprites,
    board::BoardClear,
    cauldron::{Cauldron, FIRE_BOOST_TIME},
    drag::DragGroup,
    render::{NoRescale, ZIndex, WINDOW_SIZE},
    tween::{get_move_anim, get_relative_move_anim, FadeHierarchyBundle, TweenDoneAction},
    GameState,
};
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_interact_2d::{
    drag::{Draggable, Dragged},
    Interactable, InteractionState,
};
use iyes_loopless::{
    condition::IntoConditionalExclusiveSystem,
    prelude::{AppLooplessStateExt, ConditionSet},
};
use std::time::Duration;

pub struct CardPlugin;
impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CardEffect>()
        .add_system_to_stage(CoreStage::Last, drop_card) // run after update to get precise dragged.origin
        .add_system(place_card)
            .add_system_to_stage(CoreStage::PostUpdate,shift_cards) // works with removedComponents, so can't run during Last
            ;

        if cfg!(debug_assertions) {
            app.add_enter_system(GameState::Playing, test_card_spawn);
        }
    }
}

pub const MAX_CARDS: usize = 4;
pub const CARD_SIZE: Vec2 = Vec2::new(32., 48.);
const CARD_INDEX_X_OFFSET: f32 = -145.;
const CARD_TWEEN_OFFSET: f32 = 250.;

#[derive(Component, Inspectable)]
pub struct Card {}

#[derive(Component, Inspectable)]
pub struct CardInventoryIndex(usize);

#[derive(Component, Debug, Inspectable, Clone, Copy)]
pub enum Ingredient {
    Pumpkin = 1,
    Potato,
    Tomato,
    Eggplant,
    Mushroom,
    Garlic,
}

impl Ingredient {
    pub fn get_sprite_index(&self) -> usize {
        match self {
            Ingredient::Eggplant => 0,
            Ingredient::Pumpkin => 1,
            Ingredient::Potato => 2,
            Ingredient::Mushroom => 3,
            Ingredient::Garlic => 4,
            Ingredient::Tomato => 5,
        }
    }
}

pub enum CardEffect {
    FireBoost(Entity),
    Ingredient {
        ingredient: Ingredient,
        cauldron_e: Entity,
    },
}

pub fn spawn_card(cmd: &mut Commands, sprites: &Sprites, clear: &BoardClear) {
    let corner = CARD_SIZE / 2.;
    let ingredient = match clear {
        BoardClear::Row(row) => match row {
            0..=2 => Ingredient::Tomato,
            3..=5 => Ingredient::Potato,
            6..=8 => Ingredient::Pumpkin,
            _ => unimplemented!("Unknown ingredient for row {row}"),
        },
        BoardClear::Column(col) => match col {
            0..=2 => Ingredient::Pumpkin,
            3..=5 => Ingredient::Potato,
            6..=8 => Ingredient::Tomato,
            _ => unimplemented!("Unknown ingredient for column {col}"),
        },
        BoardClear::Section(section) => match section {
            0 => Ingredient::Eggplant,
            1 | 5 => Ingredient::Pumpkin,
            2 | 6 => Ingredient::Potato,
            3 | 7 => Ingredient::Tomato,
            4 => Ingredient::Mushroom,
            8 => Ingredient::Garlic,
            _ => unimplemented!("Unknown ingredient for section {section}"),
        },
    };

    cmd.spawn_bundle(SpriteBundle {
        texture: sprites.card.clone(),
        transform: Transform::from_translation(Vec3::new(
            WINDOW_SIZE.x / 2. - CARD_SIZE.x - 60.,
            WINDOW_SIZE.y / 2. - CARD_SIZE.y - 75. + CARD_TWEEN_OFFSET,
            2.,
        )),
        ..default()
    })
    .insert(ZIndex::Card)
    .insert(Card {})
    .insert(ingredient)
    .insert(Interactable {
        bounding_box: (-corner, corner),
        groups: vec![DragGroup::Card.into()],
    })
    .insert(Draggable {
        groups: vec![DragGroup::Card.into()],
        ..default()
    })
    .insert(Name::new("Card"))
    .with_children(|b| {
        b.spawn_bundle(SpriteSheetBundle {
            texture_atlas: sprites.ingredients.clone(),
            sprite: TextureAtlasSprite::new(ingredient.get_sprite_index()),
            transform: Transform::from_translation(Vec2::new(0., 10.).extend(0.0)),
            ..default()
        })
        .insert(NoRescale);
    });
}

fn test_card_spawn(mut cmd: Commands, sprites: Res<Sprites>) {
    for i in 0..4 {
        // spawn_card(&mut cmd, &sprites, &BoardClear::Section(0));
        spawn_card(&mut cmd, &sprites, &BoardClear::Section(i));
    }
}

fn place_card(
    mut cmd: Commands,
    mut new_card_q: Query<(Entity, &mut Sprite, &mut Transform), Added<Card>>,
    card_q: Query<(), With<Card>>,
) {
    let mut card_i = card_q.iter().len() - new_card_q.iter().count();

    for (c_e, mut c_sprite, mut card_t) in new_card_q.iter_mut() {
        c_sprite.color = Color::WHITE;
        let target_pos = Vec3::new(
            card_t.translation.x + CARD_INDEX_X_OFFSET * card_i as f32,
            card_t.translation.y - CARD_TWEEN_OFFSET,
            card_t.translation.z,
        );

        cmd.entity(c_e)
            .insert(CardInventoryIndex(card_i))
            .insert(get_move_anim(
                target_pos + Vec3::Y * 250.,
                target_pos,
                450,
                None,
            ));
        card_i += 1;
    }
}

fn drop_card(
    mut cmd: Commands,
    mouse_input: Res<Input<MouseButton>>,
    dragged_query: Query<(Entity, &Card, &Ingredient, &Dragged, &Transform)>,
    interaction_state: Res<InteractionState>,
    parent_q: Query<&Parent>,
    mut cauldron_q: Query<&mut Cauldron>,
    mut card_evw: EventWriter<CardEffect>,
) {
    if mouse_input.just_released(MouseButton::Left) {
        let mut used = false;

        if let Some((e, ..)) = interaction_state.get_group(DragGroup::Fire.into()).first() {
            if let Ok(cauldron_e) = parent_q.get(*e) {
                if let Ok(mut c) = cauldron_q.get_mut(cauldron_e.get()) {
                    // increase fire boost
                    let dur = c
                        .fire_boost
                        .duration()
                        .saturating_add(Duration::from_secs_f32(FIRE_BOOST_TIME));
                    c.fire_boost.set_duration(dur);
                    c.fire_boost.reset();
                    card_evw.send(CardEffect::FireBoost(cauldron_e.get()));
                    used = true;
                }
            }
        } else if let Some((e, ..)) = interaction_state
            .get_group(DragGroup::Cauldron.into())
            .first()
        {
            if let Ok(cauldron_e) = parent_q.get(*e) {
                if let Ok(mut c) = cauldron_q.get_mut(cauldron_e.get()) {
                    // there can't be a ready meal in the cauldron
                    if c.cooked.is_none() {
                        info!("cook, plz!");
                        if let Ok((_, _, ingredient, ..)) = dragged_query.get_single() {
                            c.ingredients.push(*ingredient);
                            card_evw.send(CardEffect::Ingredient {
                                cauldron_e: cauldron_e.get(),
                                ingredient: *ingredient,
                            });

                            used = true;
                        }
                    }
                }
            }
        };

        if used {
            if let Ok((e, ..)) = dragged_query.get_single() {
                // todo: particles?
                let mut cmd_e = cmd.entity(e);
                cmd_e.remove::<Interactable>();
                cmd_e.insert_bundle(
                    FadeHierarchyBundle::new(false, 300, Color::NONE)
                        .with_done_action(TweenDoneAction::DespawnRecursive),
                );
            }
        } else {
            for (dragged_e, _card, _, dragged, card_t) in dragged_query.iter() {
                let mut e_cmd = cmd.entity(dragged_e);
                e_cmd.remove::<Dragged>();
                e_cmd.insert(get_relative_move_anim(
                    dragged.origin.extend(card_t.translation.z),
                    300,
                    None,
                ));
            }
        }
    }
}

fn shift_cards(
    mut cmd: Commands,
    mut card_inventory_q: Query<(&mut CardInventoryIndex, &mut Transform, Entity)>,
    removed_cards: RemovedComponents<Card>,
) {
    if removed_cards.iter().len() > 0 {
        let used_indices: Vec<usize> = card_inventory_q.iter().map(|(i, ..)| i.0).collect();
        let card_count = card_inventory_q.iter().len();
        let lowest_free_index = (0..card_count)
            .into_iter()
            .filter(|i| !used_indices.contains(i))
            .min();

        if let Some(mut i) = lowest_free_index {
            let mut cards = card_inventory_q.iter_mut().collect::<Vec<_>>();
            cards.sort_by(|(x, ..), (y, ..)| x.0.cmp(&y.0));
            for (ref mut c_index, ref mut c_t, ref c_e) in cards.iter_mut() {
                if c_index.0 > i {
                    cmd.entity(*c_e).insert(get_relative_move_anim(
                        Vec3::new(
                            c_t.translation.x - (c_index.0 - i) as f32 * CARD_INDEX_X_OFFSET,
                            c_t.translation.y,
                            c_t.translation.z,
                        ),
                        300,
                        None,
                    ));
                    c_index.0 = i;
                    i += 1;
                }
            }
        }
    }
}
