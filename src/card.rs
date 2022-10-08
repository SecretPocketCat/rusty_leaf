use crate::{
    assets::Sprites,
    board::{Board, BoardClear},
    cauldron::{Cauldron, TooltipIngridientList},
    drag::{Draggable, Dragged},
    highlight::Highligtable,
    interaction::{Interactable, InteractionEv, InteractionGroup, InteractionState},
    level::{InteractableSection, LevelEv},
    list::{ListPlugin, ListPluginOptions},
    render::{
        ZIndex, COL_DARK, COL_LIGHT, COL_OUTLINE_HIGHLIGHTED, PADDED_VIEW_EXTENDS, VIEW_PADDING,
        VIEW_SIZE,
    },
    tween::{
        delay_tween, get_relative_move_anim, get_relative_move_by_tween, FadeHierarchyBundle,
        TweenDoneAction,
    },
    GameState,
};
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_tweening::{Animator, EaseFunction};
use iyes_loopless::prelude::*;

pub struct CardPlugin;
impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CardEffect>()
            .add_plugin(ListPlugin::<Card>::new(ListPluginOptions {
                offset: CARD_INDEX_X_OFFSET as f32,
                offscreen_offset: -CARD_SIZE.y - VIEW_PADDING,
                horizontal: true,
                place_duration_ms: 650,
                shift_duration_ms: 300,
            }))
            .add_system_to_stage(CoreStage::Last, drop_card) // run after update to get precise dragged.origin
            .add_system(on_level_over.run_not_in_state(GameState::Loading));

        if cfg!(debug_assertions) {
            app.add_system(test_card_spawn.run_in_state(GameState::Playing));
        }
    }
}

pub const MAX_CARDS: usize = 5;
pub const CARD_SIZE: Vec2 = Vec2::new(32., 48.);
pub const CARD_EXTENDS: Vec2 = Vec2::new(CARD_SIZE.x / 2., CARD_SIZE.y / 2.);
const CARD_INDEX_X_OFFSET: i32 = -35;

#[derive(Component, Inspectable)]
pub struct Card {}

#[derive(Component, Debug, Inspectable, Clone, Copy, PartialEq, Eq, Hash)]
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
    FireBoost {
        cauldron_e: Entity,
        boost_dur_multiplier: Option<f32>,
    },
    Ingredient {
        ingredient: Ingredient,
        cauldron_e: Entity,
    },
    ClearSection {
        section: usize,
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
        BoardClear::Section { section_index, .. } => match section_index {
            0 => Ingredient::Eggplant,
            1 | 5 => Ingredient::Pumpkin,
            2 | 6 => Ingredient::Potato,
            3 | 7 => Ingredient::Tomato,
            4 => Ingredient::Mushroom,
            8 => Ingredient::Garlic,
            _ => unimplemented!("Unknown ingredient for section {section_index}"),
        },
    };

    let outline_e = cmd
        .spawn_bundle(SpriteBundle {
            texture: sprites.card_outline.clone(),
            sprite: Sprite {
                color: COL_DARK,
                ..default()
            },
            ..default()
        })
        .insert(Name::new("outline"))
        .id();

    let pos = Vec3::new(
        PADDED_VIEW_EXTENDS.x - CARD_EXTENDS.x,
        VIEW_SIZE.y / 2. + CARD_EXTENDS.y,
        2.,
    );

    cmd.spawn_bundle(SpriteBundle {
        texture: sprites.card.clone(),
        transform: Transform::from_translation(pos),
        ..default()
    })
    .insert(ZIndex::Card)
    .insert(Card {})
    .insert(ingredient)
    .insert(Interactable::new_rectangle(InteractionGroup::Card, corner))
    .insert(Draggable { offset: true })
    .insert(Name::new("Card"))
    .insert(Highligtable {
        sprite_e: Some(outline_e),
        hightlight_color: COL_LIGHT,
        hover_color: COL_OUTLINE_HIGHLIGHTED,
        normal_color: COL_DARK,
        drag_groups: vec![],
    })
    .add_child(outline_e)
    .with_children(|b| {
        b.spawn_bundle(SpriteSheetBundle {
            texture_atlas: sprites.ingredients.clone(),
            sprite: TextureAtlasSprite::new(ingredient.get_sprite_index()),
            transform: Transform::from_translation(Vec2::new(0., 10.).extend(0.0)),
            ..default()
        });
    });
}

fn test_card_spawn(mut cmd: Commands, mut lvl_evr: EventReader<LevelEv>, sprites: Res<Sprites>) {
    for ev in lvl_evr.iter() {
        if let LevelEv::LevelStart = ev {
            for i in 0..4 {
                // spawn_card(&mut cmd, &sprites, &BoardClear::Column(0));
                spawn_card(
                    &mut cmd,
                    &sprites,
                    &BoardClear::Section {
                        section_index: i,
                        used_special: false,
                    },
                );
            }

            break;
        }
    }
}

fn drop_card(
    mut cmd: Commands,
    dragged_query: Query<(&Card, &Ingredient, &Transform)>,
    interaction_state: Res<InteractionState>,
    parent_q: Query<&Parent>,
    mut cauldron_q: Query<&mut Cauldron>,
    section_q: Query<&InteractableSection>,
    tooltip_q: Query<&TooltipIngridientList>,
    mut interaction_evr: EventReader<InteractionEv>,
    mut card_evw: EventWriter<CardEffect>,
    board: Res<Board>,
) {
    for ev in interaction_evr.iter() {
        if let InteractionEv::DragEnd(drag_data) = ev {
            if let Ok((card, ingredient, card_t)) = dragged_query.get(drag_data.e) {
                let mut used = false;

                if let Some(e) = interaction_state.get_first_hovered_entity(&InteractionGroup::Fire)
                {
                    if let Ok(cauldron_e) = parent_q.get(e) {
                        if let Ok(_c) = cauldron_q.get_mut(cauldron_e.get()) {
                            // increase fire boost
                            card_evw.send(CardEffect::FireBoost {
                                cauldron_e: cauldron_e.get(),
                                boost_dur_multiplier: None,
                            });
                            used = true;
                        }
                    }
                } else if let Some(e) =
                    interaction_state.get_first_hovered_entity(&InteractionGroup::Cauldron)
                {
                    if let Ok(cauldron_e) = parent_q.get(e) {
                        if let Ok(mut c) = cauldron_q.get_mut(cauldron_e.get()) {
                            // there can't be a ready meal in the cauldron
                            if let Ok((_, ingredient, ..)) = dragged_query.get_single() {
                                let mut can_use_ingredient = true;

                                if let Some(tooltip_e) = c.tooltip_e {
                                    if let Ok(tooltip) = tooltip_q.get(tooltip_e) {
                                        if tooltip.ingredients.len() >= 3
                                            && !tooltip
                                                .ingredients
                                                .contains_key(&(*ingredient as u8))
                                        {
                                            can_use_ingredient = false;
                                        }
                                    }
                                }

                                if can_use_ingredient {
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
                } else if let Some(e) =
                    interaction_state.get_first_hovered_entity(&InteractionGroup::GridSection)
                {
                    if let Ok(section) = section_q.get(e) {
                        if !board.is_section_empty(section.0) {
                            card_evw.send(CardEffect::ClearSection { section: section.0 });
                            used = true;
                        }
                    }
                };

                if used {
                    // todo: particles?
                    let mut cmd_e = cmd.entity(drag_data.e);
                    cmd_e.remove::<Interactable>();
                    cmd_e.insert_bundle(
                        FadeHierarchyBundle::new(false, 300, Color::NONE)
                            .with_done_action(TweenDoneAction::DespawnRecursive),
                    );
                } else {
                    let mut e_cmd = cmd.entity(drag_data.e);
                    e_cmd.remove::<Dragged>();
                    e_cmd.insert(get_relative_move_anim(
                        drag_data.origin.extend(card_t.translation.z),
                        300,
                        None,
                    ));
                }
            }
        }
    }
}

fn on_level_over(
    mut cmd: Commands,
    mut lvl_evr: EventReader<LevelEv>,
    card_q: Query<Entity, With<Card>>,
) {
    for ev in lvl_evr.iter() {
        if let LevelEv::LevelOver { .. } = ev {
            for (i, e) in card_q.iter().enumerate() {
                let mut e_cmd = cmd.entity(e);
                e_cmd.insert(Animator::new(delay_tween(
                    get_relative_move_by_tween(
                        Vec3::Y * CARD_SIZE.y * 1.5,
                        350,
                        EaseFunction::QuadraticIn,
                        Some(TweenDoneAction::DespawnRecursive),
                    ),
                    i as u64 * 100,
                )));

                // prevent shifting on cleanup
                e_cmd.remove::<Card>();
            }

            break;
        }
    }
}
