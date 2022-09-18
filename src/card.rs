use std::time::Duration;

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

use crate::{
    assets::Sprites,
    board::BoardClear,
    cauldron::{Cauldron, FIRE_BOOST_TIME},
    drag::DragGroup,
    render::WINDOW_SIZE,
    GameState,
};

pub struct CardPlugin;
impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Last, drop_card) // run after update to get precise dragged.origin
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

#[derive(Component, Inspectable)]
pub struct Card {}

#[derive(Component, Inspectable)]
pub struct CardInventoryIndex(usize);

#[derive(Component, Debug, Inspectable, Clone, Copy)]
pub enum Ingredient {
    Pumpkin,
    Potato,
    Tomato,
    Eggplant,
    Mushroom,
    Garlic,
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

    let sprite_index = match ingredient {
        Ingredient::Eggplant => 0,
        Ingredient::Pumpkin => 1,
        Ingredient::Potato => 2,
        Ingredient::Mushroom => 3,
        Ingredient::Garlic => 4,
        Ingredient::Tomato => 5,
    };

    cmd.spawn_bundle(SpriteBundle {
        texture: sprites.card.clone(),
        sprite: Sprite {
            color: Color::NONE,
            ..default()
        },
        transform: Transform::from_scale(Vec2::splat(4.).extend(1.0)).with_translation(Vec3::new(
            WINDOW_SIZE.x / 2. - CARD_SIZE.x - 60.,
            WINDOW_SIZE.y / 2. - CARD_SIZE.y - 75.,
            2.,
        )),
        ..default()
    })
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
            sprite: TextureAtlasSprite::new(sprite_index),
            transform: Transform::from_translation(Vec2::new(0., 10.).extend(0.0)),
            ..default()
        });
    });
}

fn test_card_spawn(mut cmd: Commands, sprites: Res<Sprites>) {
    for i in 0..4 {
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
        // todo: tween
        c_sprite.color = Color::WHITE;
        card_t.translation.x += CARD_INDEX_X_OFFSET * card_i as f32;
        cmd.entity(c_e).insert(CardInventoryIndex(card_i));
        card_i += 1;
    }
}

fn drop_card(
    mut cmd: Commands,
    mouse_input: Res<Input<MouseButton>>,
    mut dragged_query: Query<(Entity, &Card, &Ingredient, &Dragged, &mut Transform)>,
    interaction_state: Res<InteractionState>,
    parent_q: Query<&Parent>,
    mut cauldron_q: Query<&mut Cauldron>,
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
                            used = true;
                        }
                    }
                }
            }
        };

        if used {
            if let Ok((e, ..)) = dragged_query.get_single() {
                // todo: tween
                cmd.entity(e).despawn_recursive();
            }
        } else {
            for (dragged_e, _card, _, dragged, mut card_t) in dragged_query.iter_mut() {
                let mut e_cmd = cmd.entity(dragged_e);
                e_cmd.remove::<Dragged>();
                card_t.translation = dragged.origin.extend(card_t.translation.z);
                // todo: tween back
            }
        }
    }
}

fn shift_cards(
    mut card_inventory_q: Query<(&mut CardInventoryIndex, &mut Transform)>,
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
            for (ref mut c_index, ref mut c_t) in cards.iter_mut() {
                if c_index.0 > i {
                    c_t.translation.x -= (c_index.0 - i) as f32 * CARD_INDEX_X_OFFSET;
                    c_index.0 = i;
                    i += 1;
                }
            }
        }
    }
}
