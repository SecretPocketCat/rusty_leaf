use crate::{
    assets::Sprites,
    board::{Board, BoardClear, BoardClearQueue},
    card::{spawn_card, Card, MAX_CARDS},
    coords::TileCoords,
    drag::Mover,
    piece::{spawn_piece, FieldCoords, Piece, PieceFields, PlacedFieldIndex},
    GameState,
};
use bevy::prelude::*;

use bevy_interact_2d::drag::Dragged;

use iyes_loopless::prelude::*;
use rand::Rng;

pub const BOARD_SIZE_PX: f32 = 480.;
pub const BOARD_SIZE: usize = 9;
pub const TILE_SIZE: f32 = BOARD_SIZE_PX / BOARD_SIZE as f32;
pub const SECTION_SIZE: usize = 3;
pub const BOARD_SHIFT: Vec3 = Vec3::new(-362.0, -103., 0.);

pub struct TilePlacementPlugin;
impl Plugin for TilePlacementPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::new(BOARD_SIZE, BOARD_SIZE, SECTION_SIZE))
            .insert_resource(Pieces {
                pieces: vec![
                    PieceFields::new(&[0, 1], 2, BOARD_SIZE),
                    PieceFields::new(&[0, 1], 1, BOARD_SIZE),
                    PieceFields::new(&[0, 1, 2], 1, BOARD_SIZE),
                    PieceFields::new(&[0, 1, 2, 3], 2, BOARD_SIZE),
                    PieceFields::new(&[0, 1, 2], 2, BOARD_SIZE),
                    PieceFields::new(&[0, 1, 3], 2, BOARD_SIZE),
                    PieceFields::new(&[0, 1, 2, 5], 3, BOARD_SIZE),
                    PieceFields::new(&[0, 1, 2, 3], 3, BOARD_SIZE),
                    PieceFields::new(&[0, 1, 2, 4], 3, BOARD_SIZE),
                    PieceFields::new(&[1, 3, 4, 5], 3, BOARD_SIZE),
                ],
            })
            .init_resource::<BoardClearQueue>()
            .add_system(fill_piece_queue.run_not_in_state(GameState::Loading))
            .add_system_to_stage(
                CoreStage::Last,
                clear_board.run_not_in_state(GameState::Loading),
            )
            .add_system(drop_piece.run_not_in_state(GameState::Loading));
    }
}

pub struct Pieces {
    pub pieces: Vec<PieceFields>,
}

fn fill_piece_queue(mut cmd: Commands, pieces: Res<Pieces>, pieces_q: Query<Entity, With<Piece>>) {
    if pieces_q.iter().len() == 0 {
        let pieces_len = pieces.pieces.len();
        let mut rng = rand::thread_rng();
        for i in 0..3 {
            let piece_i = rng.gen_range(0..pieces_len);
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
                i * 150 + 100,
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
    dragged_query: Query<(Entity, &Piece, &TileCoords, &Mover), With<Dragged>>,
    child_q: Query<&Children>,
    field_q: Query<&FieldCoords>,
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

                    if let Ok(children) = child_q.get(mover.moved_e) {
                        for c in children.iter() {
                            if let Ok(fld_coords) = field_q.get(*c) {
                                cmd.entity(*c).insert(PlacedFieldIndex(
                                    board.tile_coords_to_tile_index(coords + fld_coords.0),
                                ));
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

fn clear_board(
    mut cmd: Commands,
    mut queue: ResMut<BoardClearQueue>,
    mut board: ResMut<Board>,
    sprites: Res<Sprites>,
    field_q: Query<(Entity, &PlacedFieldIndex)>,
    card_q: Query<&Card>,
) {
    if queue.is_changed() {
        let mut cleared_indices: Vec<usize> = Vec::default();
        let mut allowed_card_spawn_count = MAX_CARDS.saturating_sub(card_q.iter().len());
        while let Some(c) = queue.queue.pop_front() {
            if allowed_card_spawn_count > 0 {
                spawn_card(&mut cmd, &sprites, &c);
                allowed_card_spawn_count -= 1;
            }

            match c {
                BoardClear::Row(row) => cleared_indices.extend(board.clear_row(row)),
                BoardClear::Column(col) => cleared_indices.extend(board.clear_column(col)),
                BoardClear::Section(section) => {
                    cleared_indices.extend(board.clear_section(section))
                }
            }
        }

        for (e, ..) in field_q
            .iter()
            .filter(|(_, f)| cleared_indices.contains(&f.0))
        {
            cmd.entity(e).despawn_recursive();
        }
    }
}
