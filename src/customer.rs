use bevy::prelude::*;
use iyes_loopless::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    anim::SheetAnimation,
    assets::Sprites,
    level::LevelEv,
    order::{Order, OrderEv},
    render::ZIndex,
    GameState, VIEW_SIZE,
};

pub struct CustomerPlugin;
impl Plugin for CustomerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_not_in_state(GameState::Loading)
                .with_system(spawn_customer)
                .with_system(wander_around)
                .with_system(on_order_completed)
                .with_system(on_level_over)
                .into(),
        );
        // works with removedComponents, so can't run during Last;
    }
}

#[derive(Component)]
struct Customer {
    order_e: Entity,
    target_x: f32,
    speed: f32,
    character_index: usize,
}

fn spawn_customer(
    mut cmd: Commands,
    sprites: Res<Sprites>,
    new_order_q: Query<Entity, Added<Order>>,
    customer_q: Query<&Customer>,
) {
    let mut rng = thread_rng();
    let frame_duration = rng.gen_range(110..140);
    let speed = ((140. - frame_duration as f32) + 30.) / 4.;

    for e in new_order_q.iter() {
        let current_indices: Vec<_> = customer_q.iter().map(|c| c.character_index).collect();
        // todo: get row count from sheet?
        let valid_indices: Vec<_> = (0..8_usize)
            .into_iter()
            .filter(|i| !current_indices.contains(i))
            .collect();

        let character_index = if valid_indices.len() > 0 {
            valid_indices[rng.gen_range(0..valid_indices.len())]
        } else {
            warn!("Out of valid character indices");
            rng.gen_range(0..8)
        };

        let frames = 4; // todo: get from sheet at the start?

        let range_start = character_index * frames;

        cmd.spawn_bundle(SpriteSheetBundle {
            texture_atlas: sprites.characters.clone(),
            sprite: TextureAtlasSprite {
                index: character_index,
                flip_x: true,
                ..default()
            },
            transform: Transform::from_xyz(VIEW_SIZE.x / 2. + 13., -60., 0.0),
            ..default()
        })
        .insert(ZIndex::Character)
        .insert(
            SheetAnimation::new(frame_duration)
                .with_range(range_start..(range_start + frames), true),
        )
        .insert(Customer {
            order_e: e,
            target_x: get_rand_target(),
            speed,
            character_index,
        })
        .insert(Name::new("customer"));
    }
}

fn get_rand_target() -> f32 {
    thread_rng().gen_range(0.0..(VIEW_SIZE.x / 2. - 13.))
}

fn wander_around(
    mut cmd: Commands,
    mut customer_q: Query<(
        Entity,
        &mut Customer,
        &mut Transform,
        &mut TextureAtlasSprite,
    )>,
    time: Res<Time>,
) {
    for (e, mut c, mut c_t, mut sprite) in customer_q.iter_mut() {
        if c_t.translation.x > VIEW_SIZE.x / 2. + 50. {
            // despawn if offscreen
            cmd.entity(e).despawn_recursive();
        } else {
            if (c.target_x - c_t.translation.x).abs() < 5. {
                c.target_x = get_rand_target();
                sprite.flip_x = c.target_x < c_t.translation.x;
            }

            let going_left = c.target_x < c_t.translation.x;
            c_t.translation.x +=
                c.speed * time.delta_seconds() * (if going_left { -1. } else { 1. });
        }
    }
}

fn on_order_completed(
    mut customer_q: Query<(&mut Customer, &mut TextureAtlasSprite)>,
    mut order_evr: EventReader<OrderEv>,
) {
    for ev in order_evr.iter() {
        if let OrderEv::Completed(e) = ev {
            if let Some((mut c, mut sprite)) = customer_q.iter_mut().find(|(c, ..)| c.order_e == *e)
            {
                walk_away(&mut c, &mut sprite);
            }
        }
    }
}

fn on_level_over(
    mut lvl_evr: EventReader<LevelEv>,
    mut customer_q: Query<(&mut Customer, &mut TextureAtlasSprite)>,
) {
    for ev in lvl_evr.iter() {
        if let LevelEv::LevelOver { .. } = ev {
            for (mut c, mut sprite) in customer_q.iter_mut() {
                walk_away(&mut c, &mut sprite);
            }

            break;
        }
    }
}

fn walk_away(customer: &mut Customer, sprite: &mut TextureAtlasSprite) {
    // just walk offscreen
    customer.target_x = 10000.;
    sprite.flip_x = false;
}
