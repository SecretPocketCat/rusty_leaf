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
        get_relative_fade_text_anim, get_relative_move_anim, get_relative_move_by_anim,
        FadeHierarchyBundle, FadeHierarchySet,
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
            ingredient_count_range: 1..2,
            ingredient_type_range: 1..3,
            max_simultaneous_orders: 4,
            next_customer_delay_range_ms: 1000..1001,
            total_order_count: 4,
        };

        app.insert_resource(CurrentLevel::new(lvl))
            .add_event::<OrderEv>()
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Playing)
                    .with_system(spawn_orders.run_if_resource_exists::<CurrentLevel>())
                    .with_system(update_order_progress)
                    .with_system(show_order_tooltip)
                    .with_system(on_order_completed)
                    .with_system(on_order_completed)
                    .with_system(on_order_completed)
                    .into(),
            )
            .add_system(place_items::<OrderTooltip, ORDER_TOOLTIP_OFFSET, 0, false>)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                shift_items::<OrderTooltip, ORDER_TOOLTIP_OFFSET, false>,
            );
        // works with removedComponents, so can't run during Last;
    }
}

pub const ORDER_TIME_S: f32 = 60.;
pub const ORDER_DELAY_S: f32 = 0.5;
const ORDER_TOOLTIP_OFFSET: i32 = -122;

pub enum OrderEv {
    Completed(Entity),
}

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
    delay: Option<Timer>,
    tooltip_e: Option<Entity>,
}

impl Order {
    // todo: test
    pub fn is_equal(&self, ingredients: &[Ingredient]) -> bool {
        self.ingredients
            .iter()
            .all(|(i, count)| ingredients.iter().filter(|i2| i == *i2).count() as u8 == *count)
    }
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
                    delay: Some(Timer::from_seconds(ORDER_DELAY_S, false)),
                    tooltip_e: None,
                })
                .insert(Name::new("order"));
        }
    }
}

fn show_order_tooltip(
    mut cmd: Commands,
    sprites: Res<Sprites>,
    fonts: Res<Fonts>,
    mut order_q: Query<(Entity, &mut Order), Added<Order>>,
) {
    for (o_e, mut o) in order_q.iter_mut() {
        let tooltip_ingredients: Vec<_> = o
            .ingredients
            .iter()
            .enumerate()
            .map(|(i, (ingredient, count))| {
                spawn_tooltip_ingredient(*ingredient, *count, i, -6.0, &mut cmd, &sprites, &fonts).0
            })
            .collect();

        let tooltip_e = cmd
            .spawn_bundle(SpriteBundle {
                texture: sprites.order_tooltip.clone(),
                sprite: Sprite {
                    color: Color::NONE,
                    ..default()
                },
                transform: Transform::from_xyz(510., 70., 0.),
                ..default()
            })
            .insert(ZIndex::OrderTooltip)
            .insert(TooltipProgress::new(-1.5))
            .insert_bundle(FadeHierarchyBundle::new(true, 450, OUTLINE_COL))
            .insert(OrderTooltip)
            .insert(Name::new("order_tooltip"))
            .push_children(&tooltip_ingredients)
            .add_child(o_e)
            .id();

        o.tooltip_e = Some(tooltip_e);
    }
}

fn update_order_progress(
    mut order_q: Query<&mut Order>,
    mut progress_q: Query<&mut TooltipProgress>,
    time: Res<Time>,
) {
    for mut o in order_q.iter_mut().filter(|o| o.tooltip_e.is_some()) {
        if let Some(tooltip_e) = o.tooltip_e {
            // initial delay before the actual timed progress starts
            if let Some(delay) = &mut o.delay {
                delay.tick(time.delta());
                if delay.just_finished() {
                    o.delay = None;
                }
            } else {
                o.timer.tick(time.delta());
                if o.timer.just_finished() {
                    // todo: game over
                    info!("Game over!");
                } else {
                    if let Ok(mut progress) = progress_q.get_mut(tooltip_e) {
                        progress.value = o.timer.percent();
                    }
                }
            }
        }
    }
}

fn on_order_completed(
    mut cmd: Commands,
    mut order_evr: EventReader<OrderEv>,
    order_q: Query<(Entity, &Parent)>,
) {
    for ev in order_evr.iter() {
        match ev {
            OrderEv::Completed(o_e) => {
                if let Ok((o_e, o_p)) = order_q.get(*o_e) {
                    cmd.entity(o_e).despawn_recursive();
                    cmd.entity(o_p.get()).insert(get_relative_move_by_anim(
                        Vec3::X * 250.,
                        300,
                        Some(crate::tween::TweenDoneAction::DespawnRecursive),
                    ));
                }
            }
        }
    }
}
