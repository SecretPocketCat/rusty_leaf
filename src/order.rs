use crate::{
    anim::SheetAnimation,
    assets::{Fonts, Sprites},
    card::{CardEffect, Ingredient},
    cauldron::{spawn_tooltip_ingredient, TooltipIngridientList},
    drag::DragGroup,
    list::{place_items, shift_items},
    progress::TooltipProgress,
    render::{NoRescale, ZIndex, OUTLINE_COL, SCALE_MULT},
    tween::{
        get_relative_fade_text_anim, get_relative_move_anim, FadeHierarchyBundle, FadeHierarchySet,
    },
    GameState,
};
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_interact_2d::Interactable;
use iyes_loopless::prelude::*;
use rand::{thread_rng, Rng};
use std::{mem, ops::Range, time::Duration};

pub struct OrderPlugin;
impl Plugin for OrderPlugin {
    fn build(&self, app: &mut App) {
        // let lvl = Level {
        //     name: "Test".into(),
        //     allowed_ingredients: vec![Ingredient::Pumpkin, Ingredient::Potato, Ingredient::Tomato],
        //     ingredient_count_range: 1..4,
        //     ingredient_type_range: 1..3,
        //     max_simultaneous_orders: 2,
        //     next_customer_delay_range_ms: 10000..15000,
        //     total_order_count: 4,
        // };

        let lvl = Level {
            name: "Test".into(),
            allowed_ingredients: vec![Ingredient::Pumpkin, Ingredient::Potato, Ingredient::Tomato],
            ingredient_count_range: 8..12,
            ingredient_type_range: 3..4,
            max_simultaneous_orders: 4,
            next_customer_delay_range_ms: 1000..1001,
            total_order_count: 4,
        };

        app.insert_resource(CurrentLevel::new(lvl))
            .add_system(
                spawn_orders
                    .run_in_state(GameState::Playing)
                    .run_if_resource_exists::<CurrentLevel>(),
            )
            .add_system(show_order_tooltip.run_in_state(GameState::Playing))
            .add_system(place_items::<OrderTooltip, ORDER_TOOLTIP_OFFSET, 0, false>)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                shift_items::<OrderTooltip, ORDER_TOOLTIP_OFFSET, false>,
            );
        // works with removedComponents, so can't run during Last;
    }
}

pub const ORDER_TIME_S: f32 = 60.;
const ORDER_TOOLTIP_OFFSET: i32 = -122;

#[derive(Clone)]
pub struct Level {
    pub name: String,
    pub max_simultaneous_orders: u8,
    total_order_count: u8,
    allowed_ingredients: Vec<Ingredient>,
    ingredient_count_range: Range<u8>,
    ingredient_type_range: Range<u8>,
    next_customer_delay_range_ms: Range<u64>,
    // pub specific_order // todo - last lvl or smt
}

pub struct CurrentLevel {
    opts: Level,
    start_timer: Option<Timer>,
    next_customer_timer: Timer,
    order_count: usize,
}

impl CurrentLevel {
    fn new(level: Level) -> Self {
        Self {
            opts: level,
            start_timer: None,
            // start_timer: Some(Timer::from(5.)),
            next_customer_timer: Timer::from_seconds(0., false),
            order_count: 0,
        }
    }
}

#[derive(Debug, Component)]
pub struct Order {
    ingredients: HashMap<Ingredient, u8>,
    timer: Timer,
}

#[derive(Component)]
pub struct OrderTooltip;

fn spawn_orders(
    mut cmd: Commands,
    mut lvl: ResMut<CurrentLevel>,
    time: Res<Time>,
    order_q: Query<(), With<Order>>,
) {
    let order_count = order_q.iter().len();
    if let Some(ref mut timer) = lvl.start_timer {
        timer.tick(time.delta());

        if timer.finished() {
            lvl.start_timer = None;
        }
    } else if order_count >= lvl.opts.max_simultaneous_orders as usize {
        // bail out if there're too many orders
        return;
    } else if lvl.order_count < (lvl.opts.total_order_count as usize) {
        lvl.next_customer_timer.tick(time.delta());

        if lvl.next_customer_timer.finished() {
            let mut rng = thread_rng();

            // setup next timer
            let delay = rng.gen_range(lvl.opts.next_customer_delay_range_ms.clone());
            lvl.next_customer_timer
                .set_duration(Duration::from_millis(delay));
            lvl.next_customer_timer.reset();

            // create order
            lvl.order_count += 1;

            let ingredient_count = rng.gen_range(lvl.opts.ingredient_count_range.clone());
            let type_count = rng.gen_range(lvl.opts.ingredient_type_range.clone());
            let mut ingredient_types = Vec::new();

            {
                let mut available_ingredients = lvl.opts.allowed_ingredients.clone();
                for _ in 0..type_count {
                    ingredient_types.push(
                        available_ingredients
                            .swap_remove(rng.gen_range(0..available_ingredients.len())),
                    );
                }
            }

            let mut ingredients = HashMap::new();
            for _ in 0..ingredient_count {
                let i = ingredient_types[rng.gen_range(0..ingredient_types.len())];

                if let Some(count) = ingredients.get_mut(&i) {
                    *count += 1;
                } else {
                    ingredients.insert(i, 1);
                }
            }

            cmd.spawn()
                .insert(Order {
                    ingredients,
                    timer: Timer::from_seconds(ORDER_TIME_S, false),
                })
                .insert(Name::new("order"));
        }
    }
}

fn show_order_tooltip(
    mut cmd: Commands,
    sprites: Res<Sprites>,
    fonts: Res<Fonts>,
    order_q: Query<&Order, Added<Order>>,
) {
    for o in order_q.iter() {
        let tooltip_ingredients: Vec<_> = o
            .ingredients
            .iter()
            .enumerate()
            .map(|(i, (ingredient, count))| {
                spawn_tooltip_ingredient(*ingredient, *count, i, -6.0, &mut cmd, &sprites, &fonts).0
            })
            .collect();

        cmd.spawn_bundle(SpriteBundle {
            texture: sprites.order_tooltip.clone(),
            sprite: Sprite {
                color: Color::NONE,
                ..default()
            },
            transform: Transform::from_xyz(510., 70., 0.),
            ..default()
        })
        .insert(ZIndex::OrderTooltip)
        .insert(TooltipProgress::new())
        .insert_bundle(FadeHierarchyBundle::new(true, 450, OUTLINE_COL))
        .insert(OrderTooltip)
        .insert(Name::new("order_tooltip"))
        .push_children(&tooltip_ingredients);
    }
}
