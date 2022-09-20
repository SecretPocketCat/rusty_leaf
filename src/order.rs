use crate::{
    assets::{Fonts, Sprites},
    card::Ingredient,
    cauldron::spawn_tooltip_ingredient,
    level::{CurrentLevel, LevelEv, Levels},
    list::{place_items, shift_items},
    progress::TooltipProgress,
    render::{ZIndex, COL_DARK},
    tween::{get_relative_move_by_anim, FadeHierarchyBundle},
    GameState,
};
use bevy::{prelude::*, utils::HashMap};
use iyes_loopless::prelude::*;
use rand::{thread_rng, Rng};
use std::{ops::Range, time::Duration};

pub struct OrderPlugin;
impl Plugin for OrderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OrderEv>()
            // todo: try to restore last reached lvl
            // .insert_resource(CurrentLevel::new(0))
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Playing)
                    .with_system(spawn_orders)
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

pub const ORDER_TIME_S: f32 = 3.;
// pub const ORDER_TIME_S: f32 = 80.;
pub const ORDER_DELAY_S: f32 = 0.5;
const ORDER_TOOLTIP_OFFSET: i32 = -122;

pub enum OrderEv {
    Completed(Entity),
}

pub struct SpecialOrder {
    pub index_range: Range<u8>,
    pub ingredients: HashMap<Ingredient, u8>,
}

#[derive(Debug, Component)]
pub struct Order {
    ingredients: HashMap<Ingredient, u8>,
    timer: Timer,
    delay: Option<Timer>,
    tooltip_e: Option<Entity>,
}

impl Order {
    pub fn is_equal(&self, ingredients: &[Ingredient]) -> bool {
        self.ingredients.values().sum::<u8>() == ingredients.len() as u8
            && self
                .ingredients
                .iter()
                .all(|(i, count)| ingredients.iter().filter(|i2| i == *i2).count() as u8 == *count)
    }
}
#[derive(Component)]
pub struct OrderTooltip;

fn spawn_orders(
    mut cmd: Commands,
    lvls: Res<Levels>,
    mut lvl: ResMut<CurrentLevel>,
    time: Res<Time>,
    order_q: Query<(), With<Order>>,
    mut order_evw: EventWriter<LevelEv>,
) {
    if lvl.stopped {
        return;
    }

    let lvl_opts = &lvls[lvl.level_index];

    let active_order_count = order_q.iter().len();
    if let Some(ref mut timer) = lvl.start_timer {
        timer.tick(time.delta());

        if timer.finished() {
            lvl.start_timer = None;
        }
    } else if active_order_count >= lvl_opts.max_simultaneous_orders as usize {
        // bail out if there're too many orders
    } else if lvl.order_count < (lvl_opts.total_order_count as usize) {
        lvl.next_customer_timer.tick(time.delta());

        if lvl.next_customer_timer.finished() || active_order_count == 0 {
            let mut rng = thread_rng();

            // setup next timer
            let delay = rng.gen_range(lvl_opts.next_customer_delay_range_ms.clone());
            lvl.next_customer_timer
                .set_duration(Duration::from_millis(delay));
            lvl.next_customer_timer.reset();

            // create order
            lvl.order_count += 1;

            let ingredient_count = rng.gen_range(lvl_opts.ingredient_count_range.clone());
            let type_count = rng.gen_range(lvl_opts.ingredient_type_range.clone());
            let mut ingredient_types = Vec::new();

            {
                let mut available_ingredients = lvl_opts.allowed_ingredients.clone();
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
    } else if lvl.order_count >= (lvl_opts.total_order_count as usize) && active_order_count == 0 {
        // orders are done
        info!("Next lvl");
        // todo handle victory screen/thx for playing
        // todo: clear board
        // todo: permanently store last reached lvl
        order_evw.send(LevelEv::LevelOver { won: true });

        // todo: only when next lvl has started
        // cmd.insert_resource(CurrentLevel::new(lvl.level_index + 1));
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
            .insert_bundle(FadeHierarchyBundle::new(true, 450, COL_DARK))
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
    mut order_evw: EventWriter<LevelEv>,
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
                    info!("Game over!");
                    order_evw.send(LevelEv::LevelOver { won: false });
                } else if let Ok(mut progress) = progress_q.get_mut(tooltip_e) {
                    progress.value = o.timer.percent();
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
        if let OrderEv::Completed(o_e) = ev {
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(vec![(Ingredient::Tomato, 1)], vec![Ingredient::Tomato] => true)]
    #[test_case(vec![(Ingredient::Tomato, 2)], vec![Ingredient::Tomato, Ingredient::Tomato] => true)]
    #[test_case(vec![(Ingredient::Tomato, 2), (Ingredient::Potato, 1)], vec![Ingredient::Tomato, Ingredient::Potato, Ingredient::Tomato] => true)]
    #[test_case(vec![(Ingredient::Tomato, 2)], vec![Ingredient::Tomato] => false)]
    #[test_case(vec![(Ingredient::Tomato, 1)], vec![Ingredient::Tomato, Ingredient::Tomato] => false)]
    #[test_case(vec![(Ingredient::Tomato, 2), (Ingredient::Potato, 1)], vec![Ingredient::Tomato, Ingredient::Potato] => false)]
    #[test_case(vec![(Ingredient::Tomato, 1)], vec![Ingredient::Tomato, Ingredient::Potato] => false)]
    fn is_equal(
        order_ingredients: Vec<(Ingredient, u8)>,
        flat_ingredient_list: Vec<Ingredient>,
    ) -> bool {
        let order = Order {
            ingredients: order_ingredients.into_iter().collect(),
            delay: None,
            timer: Timer::default(),
            tooltip_e: None,
        };

        order.is_equal(&flat_ingredient_list)
    }
}
