use std::ops::Range;

use crate::{
    anim::SheetAnimation,
    assets::{Fonts, Sprites},
    card::Ingredient,
    order::{SpecialOrder, ORDER_TIME_S},
    render::{ZIndex, COL_DARK, COL_LIGHT, SCALE_MULT},
    tile_placement::BOARD_SHIFT,
    tools::enum_variant_eq,
    tween::{
        delay_tween, get_relative_fade_text_anim, get_relative_fade_text_tween,
        get_relative_move_by_anim, get_relative_move_by_tween, get_relative_move_tween,
    },
    GameState,
};
use bevy::prelude::*;
use bevy_tweening::{Animator, EaseFunction};
use iyes_loopless::prelude::*;
use rand::{thread_rng, Rng};

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
            },
            Level {
                name: "Smells like Halloween".into(),
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
                total_order_count: 4,
                special_order: None,
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
                total_order_count: 5,
                special_order: None,
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
                    duration_s: ORDER_TIME_S * 1.5,
                }),
            },
        ];

        app.add_event::<LevelEv>()
            // todo: restore from somewhere
            .insert_resource(Levels(levels))
            .add_exit_system(GameState::Loading, setup_app)
            .add_enter_system(GameState::Playing, on_level_in)
            .add_exit_system(GameState::Playing, on_level_out)
            .add_system(start_day.run_if_resource_exists::<StartDayDelay>())
            .add_system(on_level_over)
            .add_system(tween_on_level_ev);
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
    // todo:
    pub required_ingredients: Vec<Ingredient>,
    pub ingredient_count_range: Range<u8>,
    pub ingredient_type_range: Range<u8>,
    pub next_customer_delay_range_ms: Range<u64>,
    // todo:
    pub special_order: Option<SpecialOrder>,
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
}

impl CurrentLevel {
    pub fn new(level_index: usize, retry: bool) -> Self {
        Self {
            level_index,
            start_timer: Some(Timer::from_seconds(1.1, false)),
            next_customer_timer: Timer::from_seconds(0., false),
            order_count: 0,
            stopped: true,
            retry,
            special_order_index: None,
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
pub struct LevelEvTween {
    in_event: LevelEv,
    duration_in: u64,
    duration_out: u64,
    out_event: LevelEv,
    delay_in: u64,
    delay_out: u64,
    ease_in: EaseFunction,
    ease_out: EaseFunction,
    tween_type: LevelEventTweenType,
}

impl LevelEvTween {
    pub fn new(
        tween_type: LevelEventTweenType,
        in_event: LevelEv,
        out_event: LevelEv,
        duration: u64,
    ) -> Self {
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

fn setup_app(mut cmd: Commands, sprites: Res<Sprites>, fonts: Res<Fonts>) {
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
        LevelEvTween::new(
            LevelEventTweenType::MoveByY(910.),
            LevelEv::LevelStart,
            LevelEv::LevelOut,
            800,
        )
        .with_delay_in(500)
        .with_ease_in(EaseFunction::CircularOut),
    )
    .insert(Name::new("Parchment"));

    cmd.spawn_bundle(SpriteBundle {
        texture: sprites.title_tooltip.clone(),
        transform: Transform::from_xyz(0., 450., 0.),
        ..default()
    })
    .insert(ZIndex::Tooltip)
    .insert(
        LevelEvTween::new(
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
        LevelEvTween::new(
            LevelEventTweenType::FadeText(COL_LIGHT),
            LevelEv::LevelIn,
            LevelEv::LevelStart,
            500,
        )
        .with_delay_in(1500)
        .with_ease_in(EaseFunction::QuadraticInOut),
    )
    .insert(Name::new("continue_text"));
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
    lvls: ResMut<Levels>,
) {
    for ev in lvl_evr.iter() {
        if let LevelEv::LevelOver { won } = ev {
            if *won {
                if lvl.level_index >= lvls.len() - 1 {
                    // restart current lvl if the player wants to go again
                    cmd.insert_resource(CurrentLevel::new(0, false));
                    cmd.insert_resource(NextState::<GameState>(GameState::Won));
                } else {
                    // next lvl
                    cmd.insert_resource(CurrentLevel::new(lvl.level_index + 1, false));
                    cmd.insert_resource(NextState::<GameState>(GameState::Playing));
                }
            } else {
                cmd.insert_resource(CurrentLevel::new(lvl.level_index, true));
                cmd.insert_resource(NextState::<GameState>(GameState::Playing));
            }

            break;
        }
    }
}

fn on_level_out(mut lvl_evw: EventWriter<LevelEv>) {
    lvl_evw.send(LevelEv::LevelOut);
}

fn tween_on_level_ev(
    mut cmd: Commands,
    mut lvl_evr: EventReader<LevelEv>,
    tween_q: Query<(Entity, &LevelEvTween)>,
) {
    for ev in lvl_evr.iter() {
        for (tween_e, tween) in tween_q.iter() {
            let is_in = tween.in_event.eq(ev);

            if is_in || tween.out_event.eq(ev) {
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
