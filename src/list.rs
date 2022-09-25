use crate::tween::{get_move_anim, get_relative_move_anim};
use bevy::prelude::*;
use bevy_tweening::EaseFunction;
use std::{marker::PhantomData, time::Duration};

pub struct ListPlugin<T: Component> {
    options: ListPluginOptions,

    _t: PhantomData<T>,
}

impl<T: Component> Plugin for ListPlugin<T> {
    fn build(&self, app: &mut App) {
        app.insert_resource(ListOptions::<T> {
            options: self.options.clone(),
            _t: Default::default(),
        })
        .insert_resource(ListTweenQueue::<T> {
            place_queue: Default::default(),
            shift: Default::default(),
            timer: None,
            _t: PhantomData::<T>::default(),
        })
        .add_system(enqueue_place_items::<T>)
        // works with removedComponents, so can't run during Last
        .add_system_to_stage(CoreStage::PostUpdate, enqueue_shift_items::<T>)
        .add_system(run_queue_timer::<T>)
        .add_system(shift_items::<T>)
        .add_system(place_items::<T>.after(shift_items::<T>));
    }
}

impl<T: Component> ListPlugin<T> {
    pub fn new(options: ListPluginOptions) -> Self {
        Self {
            options,
            _t: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct ListPluginOptions {
    pub offset: f32,
    pub offscreen_offset: f32,
    pub horizontal: bool,
}

struct ListOptions<T: Component> {
    options: ListPluginOptions,
    _t: PhantomData<T>,
}

struct ListTweenQueue<T: Component> {
    timer: Option<Timer>,
    place_queue: Vec<Entity>,
    shift: bool,

    _t: PhantomData<T>,
}

#[derive(Component)]
pub struct ListIndex<T>(usize, PhantomData<T>);

impl<T> ListIndex<T> {
    pub fn new(index: usize) -> Self {
        Self(index, PhantomData::default())
    }
}

fn enqueue_place_items<T: Component>(
    mut new_item_q: Query<Entity, Added<T>>,
    mut queue: ResMut<ListTweenQueue<T>>,
) {
    for e in new_item_q.iter_mut() {
        queue.place_queue.push(e);
    }
}

fn enqueue_shift_items<T: Component>(
    removed_items: RemovedComponents<T>,
    mut queue: ResMut<ListTweenQueue<T>>,
) {
    if removed_items.iter().len() > 0 {
        queue.shift = true;
    }
}

fn run_queue_timer<T: Component>(mut queue: ResMut<ListTweenQueue<T>>, time: Res<Time>) {
    if let Some(timer) = &mut queue.timer {
        timer.tick(time.delta());

        if timer.just_finished() {
            queue.timer = None;
        }
    }
}

fn shift_items<T: Component>(
    mut cmd: Commands,
    mut index_q: Query<(&mut ListIndex<T>, &mut Transform, Entity)>,
    mut queue: ResMut<ListTweenQueue<T>>,
    opts: Res<ListOptions<T>>,
) {
    if queue.timer.is_none() {
        if queue.shift {
            let duration = 300;
            queue.timer = Some(Timer::new(Duration::from_millis(duration), false));
            queue.shift = false;

            let used_indices: Vec<usize> = index_q.iter().map(|(i, ..)| i.0).collect();
            let item_count = index_q.iter().len();
            let lowest_free_index = (0..item_count)
                .into_iter()
                .filter(|i| !used_indices.contains(i))
                .min();

            if let Some(mut i) = lowest_free_index {
                let mut items = index_q.iter_mut().collect::<Vec<_>>();
                items.sort_by(|(x, ..), (y, ..)| x.0.cmp(&y.0));
                for (ref mut c_index, ref mut c_t, ref c_e) in items.iter_mut() {
                    if c_index.0 > i {
                        let target_pos = get_item_target_position(
                            c_t.translation,
                            c_index.0 - i,
                            -opts.options.offset,
                            None,
                            opts.options.horizontal,
                        );
                        cmd.entity(*c_e)
                            .insert(get_relative_move_anim(target_pos, duration, None));
                        c_index.0 = i;
                        i += 1;
                    }
                }
            }
        }
    }
}

fn place_items<T: Component>(
    mut cmd: Commands,
    mut item_q: Query<(Entity, &mut Sprite, &mut Transform), With<T>>,
    mut queue: ResMut<ListTweenQueue<T>>,
    opts: Res<ListOptions<T>>,
) {
    // prioritize shifting over placing items
    if queue.timer.is_none() && !queue.shift && queue.place_queue.len() > 0 {
        let duration = 400;
        queue.timer = Some(Timer::new(Duration::from_millis(duration), false));

        let mut item_i = item_q.iter().len() - queue.place_queue.len();

        for e in queue.place_queue.drain(..) {
            if let Ok((c_e, mut c_sprite, mut item_t)) = item_q.get_mut(e) {
                let target_pos = get_item_target_position(
                    item_t.translation,
                    item_i,
                    opts.options.offset,
                    Some(opts.options.offscreen_offset),
                    opts.options.horizontal,
                );

                let start_pos = target_pos
                    + if opts.options.horizontal {
                        Vec3::Y
                    } else {
                        Vec3::X
                    } * 63.;

                c_sprite.color = Color::WHITE;
                item_t.translation = start_pos;

                cmd.entity(c_e)
                    .insert(ListIndex::<T>::new(item_i))
                    .insert(get_move_anim(
                        start_pos,
                        target_pos,
                        duration,
                        EaseFunction::CircularOut,
                        None,
                    ));
                item_i += 1;
            }
        }
    }
}

fn get_item_target_position(
    current_position: Vec3,
    item_index: usize,
    offset: f32,
    offscreen_offset: Option<f32>,
    is_horizontal: bool,
) -> Vec3 {
    let offscreen_offset = offscreen_offset.unwrap_or(0.);

    if is_horizontal {
        Vec3::new(
            current_position.x + offset * item_index as f32,
            current_position.y - offscreen_offset,
            current_position.z,
        )
    } else {
        Vec3::new(
            current_position.x - offscreen_offset,
            current_position.y + offset * item_index as f32,
            current_position.z,
        )
    }
}
