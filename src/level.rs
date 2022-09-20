use std::ops::Range;

use crate::{
    anim::SheetAnimation,
    assets::Sprites,
    card::Ingredient,
    order::SpecialOrder,
    render::ZIndex,
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
use iyes_loopless::prelude::AppLooplessStateExt;

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        // titles
        // Soup 101
        // Starter
        // Smells like Halloween
        // Hot potato
        // Turning up the heat
        // A Recipe for Disaster
        // Souped up

        // let lvl = Level {
        //     name: "Test".into(),
        //     allowed_ingredients: vec![Ingredient::Pumpkin, Ingredient::Potato, Ingredient::Tomato],
        //     ingredient_count_range: 1..4,
        //     ingredient_type_range: 1..3,
        //     max_simultaneous_orders: 2,
        //     next_customer_delay_range_ms: 10000..15000,
        //     total_order_count: 4,
        // };

        let levels = vec![
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
                max_simultaneous_orders: 2,
                next_customer_delay_range_ms: 10000..15000,
                total_order_count: 5,
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
                next_customer_delay_range_ms: 10000..15000,
                total_order_count: 6,
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
                next_customer_delay_range_ms: 9000..14000,
                total_order_count: 7,
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
                next_customer_delay_range_ms: 8000..13000,
                total_order_count: 9,
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
                next_customer_delay_range_ms: 7000..12000,
                total_order_count: 9,
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
                next_customer_delay_range_ms: 10000..15000,
                total_order_count: 10,
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
                next_customer_delay_range_ms: 5000..7000,
                total_order_count: 15,
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
                next_customer_delay_range_ms: 10000..15000,
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
                next_customer_delay_range_ms: 5000..7000,
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
            },
        ];

        // let test_lvl = Level {
        //     name: "Soup 101".into(),
        //     allowed_ingredients: vec![Ingredient::Pumpkin, Ingredient::Potato, Ingredient::Tomato],
        //     ingredient_count_range: 1..2,
        //     ingredient_type_range: 1..3,
        //     max_simultaneous_orders: 4,
        //     next_customer_delay_range_ms: 1000..1001,
        //     total_order_count: 4,
        // };

        app.add_event::<LevelEv>()
            .insert_resource(Levels(levels))
            .add_exit_system(GameState::Loading, setup_app)
            .add_enter_system(GameState::Playing, setup_lvl)
            .add_exit_system(GameState::Playing, teardown_lvl)
            .add_system(tween_on_level_ev);
    }
}

pub enum LevelEv {
    LevelIn,
    LevelStart,
    LevelOver(usize),
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
    pub over: bool,
}

impl CurrentLevel {
    fn new(level_index: usize) -> Self {
        Self {
            level_index,
            start_timer: None,
            // start_timer: Some(Timer::from(5.)),
            next_customer_timer: Timer::from_seconds(0., false),
            order_count: 0,
            over: false,
        }
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

fn setup_app(mut cmd: Commands, sprites: Res<Sprites>) {
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

    let pos = Vec3::new(BOARD_SHIFT.x + 25., -1500., 0.);
    cmd.spawn_bundle(SpriteBundle {
        texture: sprites.parchment.clone(),
        transform: Transform::from_translation(pos),
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
        .with_delay_in(1000)
        .with_ease_in(EaseFunction::CircularOut),
    )
    .insert(Name::new("Parchment"));
}

fn setup_lvl(mut lvl_evw: EventWriter<LevelEv>) {
    lvl_evw.send(LevelEv::LevelIn);
}

fn teardown_lvl(mut lvl_evw: EventWriter<LevelEv>) {
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
