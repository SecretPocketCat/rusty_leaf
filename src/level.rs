use std::ops::{Range, RangeBounds};

use crate::{
    anim::SheetAnimation,
    assets::{Fonts, Sprites},
    card::Ingredient,
    drag::DragGroup,
    highlight::Highligtable,
    order::SpecialOrder,
    render::{
        NoRescale, ZIndex, COL_DARK, COL_DARKER, COL_LIGHT, COL_OUTLINE_HIGHLIGHTED,
        COL_OUTLINE_HOVERED_DRAG, SCALE_MULT,
    },
    tile_placement::{Pieces, BOARD_SHIFT, BOARD_SIZE, SECTION_SIZE, TILE_SIZE},
    tools::enum_variant_eq,
    tween::{
        delay_tween, get_fade_out_sprite_anim, get_relative_fade_text_anim,
        get_relative_fade_text_tween, get_relative_move_by_anim, get_relative_move_by_tween,
        get_relative_move_tween, get_relative_sprite_color_anim, TweenDoneAction,
    },
    GameState,
};
use bevy::{ecs::event::Event, prelude::*};
use bevy_interact_2d::Interactable;
use bevy_tweening::{Animator, EaseFunction};
use iyes_loopless::prelude::*;
use rand::{distributions::WeightedIndex, thread_rng, Rng};

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        let levels = vec![
            // Level {
            //     name: "TEST".into(),
            //     allowed_ingredients: vec![Ingredient::Pumpkin],
            //     required_ingredients: Vec::new(),
            //     ingredient_count_range: 1..2,
            //     ingredient_type_range: 1..2,
            //     max_simultaneous_orders: 3,
            //     next_customer_delay_range_ms: 5000..5001,
            //     total_order_count: 3,
            //     special_order: None,
            // },
            Level {
                name: "Soup 101".into(),
                allowed_ingredients: vec![
                    Ingredient::Pumpkin,
                    Ingredient::Potato,
                    Ingredient::Tomato,
                ],
                required_ingredients: Vec::new(),
                ingredient_count_range: 1..2,
                ingredient_type_range: 1..2,
                max_simultaneous_orders: 1,
                next_customer_delay_range_ms: 20000..30000,
                total_order_count: 2,
                special_order: None,
                pieces_range: Some(0..27),
            },
            Level {
                name: "Starter".into(),
                allowed_ingredients: vec![
                    Ingredient::Pumpkin,
                    Ingredient::Potato,
                    Ingredient::Tomato,
                ],
                required_ingredients: Vec::new(),
                ingredient_count_range: 1..3,
                ingredient_type_range: 1..3,
                max_simultaneous_orders: 2,
                next_customer_delay_range_ms: 20000..30000,
                total_order_count: 3,
                special_order: None,
                pieces_range: Some(0..27),
            },
            Level {
                name: "Smells Like Halloween".into(),
                allowed_ingredients: vec![
                    Ingredient::Pumpkin,
                    Ingredient::Potato,
                    Ingredient::Tomato,
                ],
                required_ingredients: vec![Ingredient::Pumpkin],
                ingredient_count_range: 1..3,
                ingredient_type_range: 1..3,
                max_simultaneous_orders: 2,
                next_customer_delay_range_ms: 20000..30000,
                total_order_count: 5,
                special_order: None,
                pieces_range: Some(0..27),
            },
            Level {
                name: "Cutting Corners".into(),
                allowed_ingredients: vec![
                    Ingredient::Pumpkin,
                    Ingredient::Potato,
                    Ingredient::Tomato,
                ],
                required_ingredients: vec![],
                ingredient_count_range: 1..3,
                ingredient_type_range: 1..3,
                max_simultaneous_orders: 2,
                next_customer_delay_range_ms: 20000..30000,
                total_order_count: 6,
                special_order: None,
                pieces_range: Some(7..19),
            },
            Level {
                name: "Vampire's Best Friend".into(),
                allowed_ingredients: vec![
                    Ingredient::Pumpkin,
                    Ingredient::Potato,
                    Ingredient::Tomato,
                    Ingredient::Garlic,
                ],
                required_ingredients: vec![Ingredient::Garlic],
                ingredient_count_range: 1..4,
                ingredient_type_range: 1..4,
                max_simultaneous_orders: 3,
                next_customer_delay_range_ms: 20000..30000,
                total_order_count: 7,
                special_order: None,
                pieces_range: Some(0..27),
            },
            Level {
                name: "Turning Up the Heat".into(),
                allowed_ingredients: vec![
                    Ingredient::Pumpkin,
                    Ingredient::Potato,
                    Ingredient::Tomato,
                    Ingredient::Mushroom,
                    Ingredient::Garlic,
                ],
                required_ingredients: vec![],
                ingredient_count_range: 2..4,
                ingredient_type_range: 1..4,
                max_simultaneous_orders: 3,
                next_customer_delay_range_ms: 20000..30000,
                total_order_count: 7,
                special_order: None,
                pieces_range: Some(0..27),
            },
            Level {
                name: "A Recipe for Disaster".into(),
                allowed_ingredients: vec![
                    Ingredient::Pumpkin,
                    Ingredient::Potato,
                    Ingredient::Tomato,
                    Ingredient::Mushroom,
                    Ingredient::Eggplant,
                    Ingredient::Garlic,
                ],
                required_ingredients: vec![],
                ingredient_count_range: 2..5,
                ingredient_type_range: 2..4,
                max_simultaneous_orders: 4,
                next_customer_delay_range_ms: 20000..30000,
                total_order_count: 7,
                special_order: None,
                pieces_range: Some(0..35),
            },
            Level {
                name: "Fast Food".into(),
                allowed_ingredients: vec![
                    Ingredient::Pumpkin,
                    Ingredient::Potato,
                    Ingredient::Tomato,
                    Ingredient::Mushroom,
                    Ingredient::Eggplant,
                    Ingredient::Garlic,
                ],
                required_ingredients: vec![],
                ingredient_count_range: 1..2,
                ingredient_type_range: 1..2,
                max_simultaneous_orders: 4,
                next_customer_delay_range_ms: 13000..17000,
                total_order_count: 10,
                special_order: None,
                pieces_range: Some(0..7),
            },
            Level {
                name: "Souped Up".into(),
                allowed_ingredients: vec![
                    Ingredient::Pumpkin,
                    Ingredient::Potato,
                    Ingredient::Tomato,
                    Ingredient::Mushroom,
                    Ingredient::Eggplant,
                    Ingredient::Garlic,
                ],
                required_ingredients: vec![],
                ingredient_count_range: 3..6,
                ingredient_type_range: 2..4,
                max_simultaneous_orders: 4,
                next_customer_delay_range_ms: 20000..30000,
                total_order_count: 10,
                special_order: None,
                pieces_range: None,
            },
            Level {
                name: "Food Critic".into(),
                allowed_ingredients: vec![
                    Ingredient::Pumpkin,
                    Ingredient::Potato,
                    Ingredient::Tomato,
                    Ingredient::Mushroom,
                    Ingredient::Eggplant,
                    Ingredient::Garlic,
                ],
                required_ingredients: vec![],
                ingredient_count_range: 1..2,
                ingredient_type_range: 1..2,
                max_simultaneous_orders: 4,
                next_customer_delay_range_ms: 15000..20000,
                total_order_count: 15,
                special_order: Some(SpecialOrder {
                    index_range: 9..13,
                    ingredients: [
                        (Ingredient::Tomato, 3),
                        (Ingredient::Garlic, 3),
                        (Ingredient::Eggplant, 3),
                    ]
                    .into(),
                }),
                pieces_range: None,
            },
        ];

        app.add_event::<LevelEv>()
            .insert_resource(Levels(levels))
            .add_startup_system(setup_fade)
            .add_exit_system(GameState::Loading, setup_app)
            .add_enter_system(GameState::Playing, on_level_in)
            .add_exit_system(GameState::Playing, on_level_out)
            .add_system(start_day.run_if_resource_exists::<StartDayDelay>())
            .add_system(on_level_over)
            .add_system(tween_on_level_ev::<LevelEv>);
    }
}

const FAIL_MSGS: [&str; 6] = [
    "Oh no, you've lost a customer!\nWe can't have that...",
    "Don't cry over spilled milk\nand try again.",
    "Come on, use your noodle!",
    "That's a tough nut to crack.",
    "Well, aren't you in a pickle",
    "Don't be such a couch potato.",
];

#[derive(PartialEq, Eq)]
pub enum LevelEv {
    LevelIn,
    LevelStart,
    LevelOver { won: bool },
    LevelOut,
}

impl LevelEv {
    pub fn eq(&self, other: &Self) -> bool {
        enum_variant_eq(self, other)
    }
}

pub struct Level {
    pub name: String,
    pub max_simultaneous_orders: u8,
    pub total_order_count: u8,
    pub allowed_ingredients: Vec<Ingredient>,
    pub required_ingredients: Vec<Ingredient>,
    pub ingredient_count_range: Range<u8>,
    pub ingredient_type_range: Range<u8>,
    pub next_customer_delay_range_ms: Range<u64>,
    pub special_order: Option<SpecialOrder>,
    pub pieces_range: Option<Range<usize>>,
}

#[derive(Deref, DerefMut)]
pub struct Levels(Vec<Level>);

pub struct CurrentLevel {
    pub level_index: usize,
    pub start_timer: Option<Timer>,
    pub next_customer_timer: Timer,
    pub order_count: usize,
    pub stopped: bool,
    pub retry: bool,
    pub special_order_index: Option<usize>,
    pub fields_index_offset: usize,
    pub field_weights: WeightedIndex<usize>,
}

impl CurrentLevel {
    pub fn new(
        level_index: usize,
        retry: bool,
        field_weights: WeightedIndex<usize>,
        fields_index_offset: usize,
    ) -> Self {
        Self {
            level_index,
            start_timer: Some(Timer::from_seconds(1.1, false)),
            next_customer_timer: Timer::from_seconds(0., false),
            order_count: 0,
            stopped: true,
            retry,
            special_order_index: None,
            fields_index_offset,
            field_weights,
        }
    }

    pub fn has_started(&self) -> bool {
        self.start_timer.is_none()
    }
}

pub enum LevelEventTweenType {
    MoveByX(f32),
    MoveByY(f32),
    FadeText(Color),
}

#[derive(Component)]
pub struct EvTween<T: Event> {
    in_event: T,
    duration_in: u64,
    duration_out: u64,
    out_event: T,
    delay_in: u64,
    delay_out: u64,
    ease_in: EaseFunction,
    ease_out: EaseFunction,
    tween_type: LevelEventTweenType,
}

impl<T: Event> EvTween<T> {
    pub fn new(tween_type: LevelEventTweenType, in_event: T, out_event: T, duration: u64) -> Self {
        Self {
            tween_type,
            in_event,
            out_event,
            duration_in: duration,
            duration_out: duration,
            delay_in: 0,
            delay_out: 0,
            ease_in: EaseFunction::QuadraticInOut,
            ease_out: EaseFunction::QuadraticInOut,
        }
    }

    pub fn with_duration_out(mut self, out_duration: u64) -> Self {
        self.duration_out = out_duration;
        self
    }

    pub fn with_ease_in(mut self, ease: EaseFunction) -> Self {
        self.ease_in = ease;
        self
    }

    pub fn with_ease_out(mut self, ease: EaseFunction) -> Self {
        self.ease_out = ease;
        self
    }

    pub fn with_delay_in(mut self, delay: u64) -> Self {
        self.delay_in = delay;
        self
    }

    pub fn with_delay_out(mut self, delay: u64) -> Self {
        self.delay_out = delay;
        self
    }
}

#[derive(Deref, DerefMut)]
struct StartDayDelay(Timer);

#[derive(Component)]
struct LevelTooltiptext;

#[derive(Component, Deref, DerefMut)]
pub struct InteractableSection(pub usize);

#[derive(Component)]
struct StartFade;

fn setup_fade(mut cmd: Commands) {
    cmd.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::splat(4000.)),
            color: COL_DARKER,
            ..default()
        },
        transform: Transform::from_xyz(0., 0., 100.),
        ..default()
    })
    .insert(StartFade);
}

fn setup_app(
    mut cmd: Commands,
    sprites: Res<Sprites>,
    fonts: Res<Fonts>,
    fade_q: Query<Entity, With<StartFade>>,
) {
    for e in fade_q.iter() {
        cmd.entity(e).insert(get_relative_sprite_color_anim(
            Color::NONE,
            1000,
            Some(TweenDoneAction::DespawnRecursive),
        ));
    }

    for (handle, z_index, name) in [
        (sprites.bg.clone(), ZIndex::Bg, "bg"),
        (sprites.bg_shop.clone(), ZIndex::BgShop, "bg_shop"),
    ]
    .into_iter()
    {
        cmd.spawn_bundle(SpriteBundle {
            texture: handle,
            ..default()
        })
        .insert(z_index)
        .insert(Name::new(name));
    }

    for (handle, _z_index, x, y, name) in [
        (
            sprites.ferris.clone(),
            ZIndex::Shopkeep,
            -350.,
            -218.,
            "ferris",
        ),
        (
            sprites.shop_smoke.clone(),
            ZIndex::BgShop,
            -470.,
            169.,
            "shop_smoke",
        ),
    ]
    .into_iter()
    {
        cmd.spawn_bundle(SpriteSheetBundle {
            texture_atlas: handle,
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_xyz(x, y, 0.0),
            ..default()
        })
        .insert(ZIndex::Shopkeep)
        .insert(SheetAnimation::new(100))
        .insert(Name::new(name));
    }

    cmd.spawn_bundle(SpriteBundle {
        texture: sprites.parchment.clone(),
        transform: Transform::from_xyz(BOARD_SHIFT.x + 23., -1500., 0.),
        ..default()
    })
    .insert(ZIndex::Grid)
    .insert(
        EvTween::new(
            LevelEventTweenType::MoveByY(910.),
            LevelEv::LevelStart,
            LevelEv::LevelOut,
            800,
        )
        .with_delay_in(500)
        .with_ease_in(EaseFunction::CircularOut),
    )
    .insert(Name::new("Parchment"))
    .with_children(|b| {
        b.spawn_bundle(SpriteBundle {
            texture: sprites.parchment_grid.clone(),
            transform: Transform::from_xyz(0., 0., 0.5),
            ..default()
        })
        .insert(NoRescale);
    });

    cmd.spawn_bundle(SpriteBundle {
        texture: sprites.title_tooltip.clone(),
        transform: Transform::from_xyz(0., 450., 0.),
        ..default()
    })
    .insert(ZIndex::Tooltip)
    .insert(
        EvTween::new(
            LevelEventTweenType::MoveByY(-240.),
            LevelEv::LevelIn,
            LevelEv::LevelStart,
            1200,
        )
        .with_delay_in(1000)
        .with_ease_in(EaseFunction::QuadraticOut)
        .with_duration_out(800),
    )
    .insert(Name::new("lvl_title"))
    .with_children(|b| {
        b.spawn_bundle(Text2dBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font: fonts.tooltip.clone(),
                    font_size: 16.0 * SCALE_MULT,
                    color: COL_DARK,
                },
            )
            .with_alignment(TextAlignment::CENTER),
            transform: Transform::from_xyz(0., 2., 0.01)
                .with_scale(Vec2::splat(1. / SCALE_MULT).extend(1.)),
            ..default()
        })
        .insert(LevelTooltiptext);
    });

    cmd.spawn_bundle(Text2dBundle {
        text: Text::from_section(
            format!("Click anywhere to start the day..."),
            TextStyle {
                font: fonts.tooltip.clone(),
                font_size: 16.0 * SCALE_MULT,
                color: Color::NONE,
            },
        )
        .with_alignment(TextAlignment::CENTER_LEFT),
        transform: Transform::from_xyz(-50., -321., 0.),
        ..default()
    })
    .insert(ZIndex::Tooltip)
    .insert(
        EvTween::new(
            LevelEventTweenType::FadeText(COL_LIGHT),
            LevelEv::LevelIn,
            LevelEv::LevelStart,
            500,
        )
        .with_delay_in(1500)
        .with_ease_in(EaseFunction::QuadraticInOut),
    )
    .insert(Name::new("continue_text"));

    let corner = Vec2::splat(SECTION_SIZE as f32 / 2. * TILE_SIZE);
    let section_box = (-corner, corner);
    // -520., 54.
    for i in 0..(BOARD_SIZE / SECTION_SIZE).pow(2) {
        let x = (i % SECTION_SIZE) as f32 * corner.x * 2. - 520.;
        let y = (i / SECTION_SIZE) as f32 * -corner.x * 2. + 58.;

        cmd.spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(x, y, f32::from(ZIndex::Grid) + 0.1),
            sprite: Sprite {
                custom_size: Some(corner * 1.95),
                color: Color::NONE,
                ..default()
            },
            ..default()
        })
        .insert(Interactable {
            bounding_box: section_box,
            groups: vec![DragGroup::GridSection.into()],
        })
        .insert(InteractableSection(i))
        .insert(Highligtable {
            drag_groups: vec![DragGroup::Card],
            normal_color: Color::NONE,
            hightlight_color: Color::rgba(
                COL_OUTLINE_HIGHLIGHTED.r(),
                COL_OUTLINE_HIGHLIGHTED.g(),
                COL_OUTLINE_HIGHLIGHTED.b(),
                0.4,
            ),
            hover_color: Color::rgba(
                COL_OUTLINE_HOVERED_DRAG.r(),
                COL_OUTLINE_HOVERED_DRAG.g(),
                COL_OUTLINE_HOVERED_DRAG.b(),
                0.5,
            ),
            sprite_e: None,
        })
        .insert(NoRescale)
        .insert(Name::new("interactable_section"));
    }
}

fn on_level_in(
    mut cmd: Commands,
    mut lvl_evw: EventWriter<LevelEv>,
    lvl: Res<CurrentLevel>,
    lvls: Res<Levels>,
    mut title_txt_q: Query<&mut Text, With<LevelTooltiptext>>,
) {
    title_txt_q.single_mut().sections[0].value = if lvl.retry {
        FAIL_MSGS[thread_rng().gen_range(0..FAIL_MSGS.len())].into()
    } else {
        format!(
            "Day {}: {}",
            lvl.level_index + 1,
            lvls[lvl.level_index].name
        )
    };

    lvl_evw.send(LevelEv::LevelIn);
    cmd.insert_resource(StartDayDelay(Timer::from_seconds(2.15, false)));
}

fn start_day(
    mut cmd: Commands,
    mut lvl_evw: EventWriter<LevelEv>,
    mut delay: ResMut<StartDayDelay>,
    mut lvl: ResMut<CurrentLevel>,
    time: Res<Time>,
    mouse_input: Res<Input<MouseButton>>,
) {
    delay.tick(time.delta());

    if delay.finished() {
        if mouse_input.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
            cmd.remove_resource::<StartDayDelay>();
            lvl_evw.send(LevelEv::LevelStart);
            lvl.stopped = false;
        }
    }
}

fn on_level_over(
    mut cmd: Commands,
    mut lvl_evr: EventReader<LevelEv>,
    lvl: Res<CurrentLevel>,
    lvls: Res<Levels>,
    pieces: Res<Pieces>,
) {
    for ev in lvl_evr.iter() {
        if let LevelEv::LevelOver { won } = ev {
            let mut lvl_i = lvl.level_index;

            if *won {
                if lvl.level_index >= lvls.len() - 1 {
                    // restart current lvl if the player wants to go again
                    lvl_i = 0;
                    cmd.insert_resource(NextState::<GameState>(GameState::Won));
                } else {
                    // next lvl
                    lvl_i = lvl.level_index + 1;
                    cmd.insert_resource(NextState::<GameState>(GameState::Playing));
                }
            } else {
                cmd.insert_resource(NextState::<GameState>(GameState::Playing));
            }

            let range = lvls[lvl_i].pieces_range.clone();
            let dist = pieces.get_distribution(range.clone());

            cmd.insert_resource(CurrentLevel::new(
                lvl_i,
                !won,
                dist,
                range.map_or(0, |x| x.start),
            ));

            break;
        }
    }
}

fn on_level_out(mut lvl_evw: EventWriter<LevelEv>) {
    lvl_evw.send(LevelEv::LevelOut);
}

pub fn tween_on_level_ev<T: Event + Eq>(
    mut cmd: Commands,
    mut lvl_evr: EventReader<T>,
    tween_q: Query<(Entity, &EvTween<T>)>,
) {
    for ev in lvl_evr.iter() {
        for (tween_e, tween) in tween_q.iter() {
            let is_in = &tween.in_event == ev;

            if is_in || &tween.out_event == ev {
                let mut tween_e_cmd = cmd.entity(tween_e);
                let (sign, duration, delay, ease) = if is_in {
                    (1., tween.duration_in, tween.delay_in, tween.ease_in)
                } else {
                    (-1., tween.duration_out, tween.delay_out, tween.ease_out)
                };

                match tween.tween_type {
                    LevelEventTweenType::MoveByX(x) => {
                        tween_e_cmd.insert(Animator::new(delay_tween(
                            get_relative_move_by_tween(Vec3::X * x * sign, duration, ease, None),
                            delay,
                        )));
                    }
                    LevelEventTweenType::MoveByY(y) => {
                        tween_e_cmd.insert(Animator::new(delay_tween(
                            get_relative_move_by_tween(Vec3::Y * y * sign, duration, ease, None),
                            delay,
                        )));
                    }
                    LevelEventTweenType::FadeText(color) => {
                        let col = if is_in { color } else { Color::NONE };
                        tween_e_cmd.insert(Animator::new(delay_tween(
                            get_relative_fade_text_tween(col, duration, ease, None),
                            delay,
                        )));
                    }
                }
            }
        }
    }
}
