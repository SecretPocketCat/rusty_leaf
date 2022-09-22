use crate::{
    anim::SheetAnimation,
    assets::Sprites,
    board::{Board, BoardClear, BoardClearQueue},
    card::{spawn_card, Card, MAX_CARDS},
    coords::TileCoords,
    drag::Mover,
    level::{CurrentLevel, LevelEv, Levels},
    piece::{spawn_piece, FieldCoords, Piece, PieceFields, PlacedFieldIndex},
    render::ZIndex,
    tween::{
        delay_tween, get_relative_fade_spritesheet_tween, get_relative_move_by_tween,
        get_relative_move_tween, get_relative_spritesheet_color_anim, get_scale_tween,
        TweenDoneAction,
    },
    GameState,
};
use bevy::prelude::*;
use bevy_interact_2d::drag::Dragged;
use bevy_tweening::{Animator, EaseFunction};
use iyes_loopless::prelude::*;
use rand::prelude::*;
use rand::{distributions::WeightedIndex, thread_rng, Rng};
use std::ops::Range;

pub const BOARD_SIZE_PX: f32 = 480.;
pub const BOARD_SIZE: usize = 9;
pub const TILE_SIZE: f32 = BOARD_SIZE_PX / BOARD_SIZE as f32;
pub const SECTION_SIZE: usize = 3;
pub const BOARD_SHIFT: Vec3 = Vec3::new(-362.0, -103., 0.);
pub const CARDS_PER_CLEAR: usize = 2;

pub struct TilePlacementPlugin;
impl Plugin for TilePlacementPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::new(BOARD_SIZE, BOARD_SIZE, SECTION_SIZE))
            .insert_resource(Pieces::new(vec![
                // lines
                (PieceFields::new(&[0, 1], 1, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 1], 2, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 1, 2], 1, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 1, 2], 3, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 1, 2, 3], 1, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 1, 2, 3], 4, BOARD_SIZE), 10),
                // square
                (PieceFields::new(&[0, 1, 2, 3], 2, BOARD_SIZE), 10),
                // corners
                (PieceFields::new(&[0, 1, 2], 2, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 1, 3], 2, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 2, 3], 2, BOARD_SIZE), 10),
                (PieceFields::new(&[1, 2, 3], 2, BOARD_SIZE), 10),
                // L
                (PieceFields::new(&[0, 1, 2, 4], 2, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 1, 3, 5], 2, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 2, 4, 5], 2, BOARD_SIZE), 10),
                (PieceFields::new(&[1, 3, 4, 5], 2, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 3, 4, 5], 3, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 1, 2, 3], 3, BOARD_SIZE), 10),
                (PieceFields::new(&[2, 3, 4, 5], 3, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 1, 2, 4], 3, BOARD_SIZE), 10),
                // Z
                (PieceFields::new(&[1, 2, 3, 4], 3, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 1, 4, 5], 3, BOARD_SIZE), 10),
                (PieceFields::new(&[1, 2, 3, 4], 2, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 2, 3, 5], 2, BOARD_SIZE), 10),
                // T
                (PieceFields::new(&[1, 3, 4, 5], 3, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 1, 2, 4], 3, BOARD_SIZE), 10),
                (PieceFields::new(&[0, 2, 3, 4], 2, BOARD_SIZE), 10),
                (PieceFields::new(&[1, 2, 3, 5], 2, BOARD_SIZE), 10),
                // cubby
                (PieceFields::new(&[0, 1, 2, 4, 5], 2, BOARD_SIZE), 5),
                (PieceFields::new(&[0, 1, 3, 4, 5], 2, BOARD_SIZE), 5),
                (PieceFields::new(&[0, 1, 2, 3, 5], 3, BOARD_SIZE), 5),
                (PieceFields::new(&[0, 3, 4, 5], 3, BOARD_SIZE), 5),
                // edgy
                (PieceFields::new(&[0, 3], 2, BOARD_SIZE), 5),
                (PieceFields::new(&[1, 2], 2, BOARD_SIZE), 5),
                (PieceFields::new(&[0, 4, 8], 3, BOARD_SIZE), 3),
                (PieceFields::new(&[2, 4, 6], 3, BOARD_SIZE), 3),
                // cross
                (PieceFields::new(&[1, 3, 4, 5, 7], 3, BOARD_SIZE), 3),
                // edgy cross
                (PieceFields::new(&[0, 2, 4, 6, 8], 3, BOARD_SIZE), 2),
                // donut
                (
                    PieceFields::new(&[0, 1, 2, 3, 5, 6, 7, 8], 3, BOARD_SIZE),
                    1,
                ),
            ]))
            .init_resource::<BoardClearQueue>()
            .add_system(fill_piece_queue.run_in_state(GameState::Playing))
            .add_system_to_stage(
                CoreStage::Last,
                process_clear_queue.run_not_in_state(GameState::Loading),
            )
            .add_system(drop_piece.run_not_in_state(GameState::Loading))
            .add_system(on_level_over.run_in_state(GameState::Playing));
    }
}

pub struct Pieces {
    pub pieces: Vec<PieceFields>,
    distribution: Vec<usize>,
}

impl Pieces {
    pub fn new(weighted_pieces: Vec<(PieceFields, usize)>) -> Self {
        Self {
            pieces: weighted_pieces.iter().map(|x| x.0.clone()).collect(),
            distribution: weighted_pieces.iter().map(|x| x.1.clone()).collect(),
        }
    }

    pub fn get_distribution(&self, range: Option<Range<usize>>) -> WeightedIndex<usize> {
        match range {
            Some(range) => WeightedIndex::new(self.distribution[range].iter().cloned()).unwrap(),
            None => WeightedIndex::new(self.distribution.clone()).unwrap(),
        }
    }
}

pub fn spawn_tile_explosion(cmd: &mut Commands, sprites: &Sprites, position: Vec3, delay_ms: u64) {
    let mut rng = thread_rng();
    cmd.spawn_bundle(SpriteSheetBundle {
        texture_atlas: sprites.explosion.clone(),
        sprite: TextureAtlasSprite {
            flip_x: rng.gen(),
            flip_y: rng.gen(),
            color: Color::NONE,
            ..default()
        },
        transform: Transform::from_translation(position),
        ..default()
    })
    .insert(ZIndex::Explosion)
    .insert(
        SheetAnimation::new(65)
            .with_despawn_on_completion()
            .with_delay(delay_ms),
    );
}

// todo: initial spawn delay
// maybe just mark the level as started after the initial wait
fn fill_piece_queue(
    mut cmd: Commands,
    pieces: Res<Pieces>,
    pieces_q: Query<Entity, With<Piece>>,
    lvl: Res<CurrentLevel>,
) {
    if !lvl.stopped && lvl.has_started() && pieces_q.iter().len() == 0 {
        let mut rng = rand::thread_rng();
        for i in 0..3 {
            let piece_i = lvl.fields_index_offset + lvl.field_weights.sample(&mut rng);
            let x = ((i as i32) - 1i32) as f32 * 180.;
            let piece = &pieces.pieces[piece_i];
            spawn_piece(
                &mut cmd,
                piece,
                piece_i,
                Vec2::new(
                    x + BOARD_SHIFT.x,
                    BOARD_SIZE_PX / 2.
                        + BOARD_SHIFT.y
                        // + (piece.get_height() as f32 * TILE_SIZE) / 2.
                        + 100.,
                ),
                i * 150,
            );
        }
    }
}

fn drop_piece(
    mut cmd: Commands,
    mouse_input: Res<Input<MouseButton>>,
    mut board: ResMut<Board>,
    mut clear_queue: ResMut<BoardClearQueue>,
    pieces: Res<Pieces>,
    sprites: Res<Sprites>,
    dragged_query: Query<(Entity, &Piece, &TileCoords, &Mover), With<Dragged>>,
    child_q: Query<&Children>,
    field_q: Query<&FieldCoords>,
    mut transform_q: Query<(&mut Transform, &GlobalTransform)>,
) {
    if mouse_input.just_released(MouseButton::Left) {
        for (dragged_e, piece, coords, mover) in dragged_query.iter() {
            let mut e_cmd = cmd.entity(dragged_e);
            e_cmd.remove::<Dragged>();

            if let Some(coords) = coords.tile_coords {
                if let Ok(cleared) = board.place_piece(
                    coords.x as usize,
                    coords.y as usize,
                    pieces.pieces[piece.0].get_fields(),
                ) {
                    e_cmd.despawn_recursive();
                    let mut rng = rand::thread_rng();

                    if let Ok(children) = child_q.get(mover.moved_e) {
                        for (i, c) in children.iter().enumerate() {
                            if let Ok(fld_coords) = field_q.get(*c) {
                                let (mut t, t_global) = transform_q.get_mut(*c).unwrap();

                                let tile_coords =
                                    board.tile_coords_to_tile_index(coords + fld_coords.0);
                                let tween_delay = i as u64 * 50;

                                cmd.spawn_bundle(SpriteSheetBundle {
                                    texture_atlas: sprites.crosses.clone(),
                                    sprite: TextureAtlasSprite {
                                        // todo: get range from atlas
                                        index: rng.gen_range(0..16),
                                        flip_x: rng.gen(),
                                        flip_y: rng.gen(),
                                        color: Color::NONE,
                                        ..default()
                                    },
                                    transform: Transform::from_translation(t_global.translation()),
                                    ..default()
                                })
                                .insert(PlacedFieldIndex(tile_coords))
                                .insert(Animator::new(delay_tween(
                                    get_relative_fade_spritesheet_tween(Color::WHITE, 350, None),
                                    tween_delay,
                                )))
                                .insert(Name::new(format!("field [{tile_coords}]")));

                                // unparent and update pos to stay where it's
                                let mut e_cmd = cmd.entity(*c);
                                e_cmd.remove::<Parent>();
                                t.translation = t_global.translation();
                                e_cmd.insert(Animator::new(delay_tween(
                                    get_scale_tween(
                                        Vec3::ONE,
                                        Vec3::ZERO,
                                        EaseFunction::QuadraticIn,
                                        400,
                                        Some(TweenDoneAction::DespawnRecursive),
                                    ),
                                    tween_delay,
                                )));
                            }
                        }

                        cmd.entity(mover.moved_e).despawn();
                    }

                    for c in cleared {
                        clear_queue.queue.push_back(c);
                    }
                }
            }
        }
    }
}

fn process_clear_queue(
    mut cmd: Commands,
    mut queue: ResMut<BoardClearQueue>,
    mut board: ResMut<Board>,
    sprites: Res<Sprites>,
    field_q: Query<(Entity, &PlacedFieldIndex, &GlobalTransform)>,
    card_q: Query<&Card>,
) {
    if queue.is_changed() {
        let mut cleared_indices: Vec<usize> = Vec::default();
        let mut allowed_card_spawn_count = MAX_CARDS.saturating_sub(card_q.iter().len());
        while let Some(c) = queue.queue.pop_front() {
            for _ in 0..CARDS_PER_CLEAR {
                if allowed_card_spawn_count > 0 {
                    spawn_card(&mut cmd, &sprites, &c);
                    allowed_card_spawn_count -= 1;
                }
            }

            match c {
                BoardClear::Row(row) => cleared_indices.extend(board.clear_row(row)),
                BoardClear::Column(col) => cleared_indices.extend(board.clear_column(col)),
                BoardClear::Section(section) => {
                    cleared_indices.extend(board.clear_section(section))
                }
            }
        }

        let mut rng = thread_rng();

        for (i, (e, _, t)) in field_q
            .iter()
            .filter(|(_, f, ..)| cleared_indices.contains(&f.0))
            .enumerate()
        {
            let delay = i as u64 * 30;
            cmd.entity(e).insert(Animator::new(delay_tween(
                get_relative_fade_spritesheet_tween(
                    Color::NONE,
                    150,
                    Some(TweenDoneAction::DespawnRecursive),
                ),
                delay,
            )));

            spawn_tile_explosion(&mut cmd, &sprites, t.translation(), delay);
        }
    }
}

fn on_level_over(
    mut cmd: Commands,
    mut lvl_evr: EventReader<LevelEv>,
    mut board: ResMut<Board>,
    field_q: Query<Entity, With<PlacedFieldIndex>>,
    piece_q: Query<(Entity, &Mover), With<Piece>>,
) {
    for ev in lvl_evr.iter() {
        if let LevelEv::LevelOver { .. } = ev {
            board.clear();

            for (i, e) in field_q.iter().enumerate() {
                cmd.entity(e).insert(Animator::new(delay_tween(
                    get_relative_fade_spritesheet_tween(
                        Color::NONE,
                        150,
                        Some(TweenDoneAction::DespawnRecursive),
                    ),
                    i as u64 * 25,
                )));
            }

            for (i, (e, mover)) in piece_q.iter().enumerate() {
                cmd.entity(e).despawn_recursive();
                cmd.entity(mover.moved_e).insert(Animator::new(delay_tween(
                    get_relative_move_by_tween(
                        Vec3::Y * 450.,
                        350,
                        EaseFunction::CircularIn,
                        Some(TweenDoneAction::DespawnRecursive),
                    ),
                    i as u64 * 100,
                )));
            }

            break;
        }
    }
}
