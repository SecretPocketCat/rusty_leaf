use crate::{
    assets::Sprites,
    board::{Board, BoardClear},
    cauldron::{Cauldron, TooltipIngridientList},
    drag::DragGroup,
    highlight::Highligtable,
    level::{InteractableSection, LevelEv},
    list::{ListPlugin, ListPluginOptions},
    render::{NoRescale, ZIndex, COL_DARK, COL_LIGHT, COL_OUTLINE_HIGHLIGHTED, WINDOW_SIZE},
    tween::{
        delay_tween, get_relative_move_anim, get_relative_move_by_tween, FadeHierarchyBundle,
        TweenDoneAction,
    },
    GameState,
};
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_interact_2d::{
    drag::{Draggable, Dragged},
    Interactable, InteractionState,
};
use bevy_tweening::{Animator, EaseFunction};
use iyes_loopless::prelude::*;

pub struct CardPlugin;
impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CardEffect>()
            .add_plugin(ListPlugin::<Card>::new(ListPluginOptions {
                offset: CARD_INDEX_X_OFFSET as f32,
                offscreen_offset: CARD_OFFSCREEN_OFFSET as f32,
                horizontal: true,
            }))
            .add_system_to_stage(CoreStage::Last, drop_card) // run after update to get precise dragged.origin
            .add_system(on_level_over.run_not_in_state(GameState::Loading));

        if cfg!(debug_assertions) {
            app.add_enter_system(GameState::Playing, test_card_spawn);
        }
    }
}

pub const MAX_CARDS: usize = 5;
pub const CARD_SIZE: Vec2 = Vec2::new(32., 48.);
const CARD_INDEX_X_OFFSET: i32 = -140;
const CARD_OFFSCREEN_OFFSET: i32 = 250;

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
        .insert(NoRescale)
        .insert(Name::new("outline"))
        .id();

    let pos = Vec3::new(
        WINDOW_SIZE.x / 2. - CARD_SIZE.x - 60.,
        WINDOW_SIZE.y / 2. - CARD_SIZE.y - 75. + CARD_OFFSCREEN_OFFSET as f32,
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
    .insert(Interactable {
        bounding_box: (-corner, corner),
        groups: vec![DragGroup::Card.into()],
    })
    .insert(Draggable {
        groups: vec![DragGroup::Card.into()],
        ..default()
    })
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
        })
        .insert(NoRescale);
    });
}

fn test_card_spawn(mut cmd: Commands, sprites: Res<Sprites>) {
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
}

fn drop_card(
    mut cmd: Commands,
    mouse_input: Res<Input<MouseButton>>,
    dragged_query: Query<(Entity, &Card, &Ingredient, &Dragged, &Transform)>,
    interaction_state: Res<InteractionState>,
    parent_q: Query<&Parent>,
    mut cauldron_q: Query<&mut Cauldron>,
    section_q: Query<&InteractableSection>,
    tooltip_q: Query<&TooltipIngridientList>,
    mut card_evw: EventWriter<CardEffect>,
    board: Res<Board>,
) {
    if mouse_input.just_released(MouseButton::Left) {
        let mut used = false;

        if interaction_state.get_group(DragGroup::Card.into()).len() > 0 {
            if let Some((e, ..)) = interaction_state.get_group(DragGroup::Fire.into()).first() {
                if let Ok(cauldron_e) = parent_q.get(*e) {
                    if let Ok(_c) = cauldron_q.get_mut(cauldron_e.get()) {
                        // increase fire boost
                        card_evw.send(CardEffect::FireBoost {
                            cauldron_e: cauldron_e.get(),
                            boost_dur_multiplier: None,
                        });
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
                        if let Ok((_, _, ingredient, ..)) = dragged_query.get_single() {
                            let mut can_use_ingredient = true;

                            if let Some(tooltip_e) = c.tooltip_e {
                                if let Ok(tooltip) = tooltip_q.get(tooltip_e) {
                                    if tooltip.ingredients.len() >= 3
                                        && !tooltip.ingredients.contains_key(&(*ingredient as u8))
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
            } else if let Some((e, ..)) = interaction_state
                .get_group(DragGroup::GridSection.into())
                .first()
            {
                if let Ok(section) = section_q.get(*e) {
                    if !board.is_section_empty(section.0) {
                        card_evw.send(CardEffect::ClearSection { section: section.0 });
                        used = true;
                    }
                }
            };
        }

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
                        Vec3::Y * 450.,
                        350,
                        EaseFunction::CircularIn,
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
