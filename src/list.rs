use crate::tween::{get_move_anim, get_relative_move_anim};
use bevy::prelude::*;
use bevy_tweening::EaseFunction;
use std::marker::PhantomData;

#[derive(Component)]
pub struct ListIndex<T>(usize, PhantomData<T>);

impl<T> ListIndex<T> {
    pub fn new(index: usize) -> Self {
        Self(index, PhantomData::default())
    }
}

pub fn place_items<
    T: Component,
    const OFFSET: i32,
    const OFFSCREEN_OFFSET: i32,
    const IS_HORIZONTAL: bool,
>(
    mut cmd: Commands,
    mut new_item_q: Query<(Entity, &mut Sprite, &mut Transform), Added<T>>,
    item_q: Query<(), With<T>>,
) {
    let mut item_i = item_q.iter().len() - new_item_q.iter().count();

    for (c_e, mut c_sprite, mut item_t) in new_item_q.iter_mut() {
        let target_pos = get_item_target_position(
            item_t.translation,
            item_i,
            OFFSET,
            Some(OFFSCREEN_OFFSET),
            IS_HORIZONTAL,
        );
        let start_pos = target_pos + if IS_HORIZONTAL { Vec3::Y } else { Vec3::X } * 250.;

        c_sprite.color = Color::WHITE;
        item_t.translation = start_pos;

        cmd.entity(c_e)
            .insert(ListIndex::<T>::new(item_i))
            .insert(get_move_anim(
                start_pos,
                target_pos,
                450,
                EaseFunction::CircularOut,
                None,
            ));
        item_i += 1;
    }
}

pub fn shift_items<T: Component, const OFFSET: i32, const IS_HORIZONTAL: bool>(
    mut cmd: Commands,
    mut index_q: Query<(&mut ListIndex<T>, &mut Transform, Entity)>,
    removed_items: RemovedComponents<T>,
) {
    if removed_items.iter().len() > 0 {
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
                        -OFFSET,
                        None,
                        IS_HORIZONTAL,
                    );
                    cmd.entity(*c_e)
                        .insert(get_relative_move_anim(target_pos, 300, None));
                    c_index.0 = i;
                    i += 1;
                }
            }
        }
    }
}

fn get_item_target_position(
    current_position: Vec3,
    item_index: usize,
    offset: i32,
    offscreen_offset: Option<i32>,
    is_horizontal: bool,
) -> Vec3 {
    let offscreen_offset = offscreen_offset.unwrap_or(0);

    if is_horizontal {
        Vec3::new(
            current_position.x + offset as f32 * item_index as f32,
            current_position.y - offscreen_offset as f32,
            current_position.z,
        )
    } else {
        Vec3::new(
            current_position.x - offscreen_offset as f32,
            current_position.y + offset as f32 * item_index as f32,
            current_position.z,
        )
    }
}
