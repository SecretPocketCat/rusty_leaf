use crate::{
    board::Board,
    coords::{get_world_coords_from_tile, TileCoords},
    mouse::CursorWorldPosition,
    piece::{spawn_piece, Piece, PieceFields},
    GameState,
};
use bevy::prelude::*;
use bevy_inspector_egui::InspectorPlugin;
use bevy_interact_2d::{
    drag::{Draggable, Dragged},
    Interactable, InteractionState,
};
use bevy_prototype_lyon::prelude::*;
use iyes_loopless::prelude::*;
use rand::Rng;
use std::{
    collections::VecDeque,
    ops::{Add, Div, Sub},
};

pub const BOARD_SIZE_PX: f32 = 540.;
pub const BOARD_SIZE: usize = 9;
pub const TILE_SIZE: f32 = BOARD_SIZE_PX / BOARD_SIZE as f32;
pub const SECTION_SIZE: usize = 3;

pub struct TilePlacementPlugin;
impl Plugin for TilePlacementPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugin(InspectorPlugin::<Board>::new())
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
                queue: default(),
            })
            .add_system_to_stage(CoreStage::Last, limit_drag_count)
            .add_enter_system(GameState::Playing, setup_board)
            .add_system(fill_piece_queue.run_if_resource_exists::<Pieces>())
            .add_system(drop_piece)
            .add_system(drag_piece)
            .add_system(process_movers);
    }
}

pub struct Pieces {
    pieces: Vec<PieceFields>,
    queue: VecDeque<usize>,
}

#[derive(Component)]
pub struct Mover {
    pub moved_e: Entity,
}

fn setup_board(mut cmd: Commands) {
    let size = BOARD_SIZE_PX;
    let extents = size / 2.;
    let square = shapes::Rectangle {
        extents: Vec2::splat(size),
        ..shapes::Rectangle::default()
    };
    let builder = GeometryBuilder::new().add(&square);

    cmd.spawn_bundle(builder.build(
        DrawMode::Fill(FillMode::color(Color::ANTIQUE_WHITE)),
        Transform::default(),
    ))
    .insert(Name::new("board_bg"));

    let mut builder = GeometryBuilder::new();

    let section_count = 9;
    let section_count_half = section_count / 2;
    let section_size = size / section_count as f32;
    for i in -section_count_half..section_count_half {
        let x = section_size * i as f32 + section_size / 2.;
        let line_x = shapes::Line(Vec2::new(x, extents), Vec2::new(x, -extents));
        let line_y = shapes::Line(Vec2::new(extents, x), Vec2::new(-extents, x));
        builder = builder.add(&line_x).add(&line_y);
    }

    cmd.spawn_bundle(builder.build(
        DrawMode::Stroke(StrokeMode::new(Color::GRAY, 5.0)),
        Transform::default(),
    ))
    .insert(Name::new("board_sections"));

    let mut builder = GeometryBuilder::new().add(&square);

    let section_count = 3;
    let section_count_half = section_count / 2;
    let section_size = size / section_count as f32;
    for i in -section_count_half..section_count_half {
        let x = section_size * i as f32 + section_size / 2.;
        let line_x = shapes::Line(Vec2::new(x, extents), Vec2::new(x, -extents));
        let line_y = shapes::Line(Vec2::new(extents, x), Vec2::new(-extents, x));
        builder = builder.add(&line_x).add(&line_y);
    }

    cmd.spawn_bundle(builder.build(
        DrawMode::Stroke(StrokeMode::new(Color::DARK_GRAY, 12.0)),
        Transform::default(),
    ))
    .insert(Name::new("board_lines"));

    cmd.insert_resource(Board::new(9, 9, 3));
}

fn fill_piece_queue(mut cmd: Commands, mut pieces: ResMut<Pieces>) {
    if pieces.is_changed() && pieces.queue.len() == 0 {
        let pieces_len = pieces.pieces.len();
        for _ in 0..3 {
            pieces
                .queue
                .push_back(rand::thread_rng().gen_range(0..pieces_len));
        }

        for (i, piece_i) in pieces.queue.iter().enumerate() {
            let x = ((i as i32) - 1i32) as f32 * 230.;
            spawn_piece(
                &mut cmd,
                pieces.pieces.get(*piece_i).unwrap(),
                *piece_i,
                Vec2::new(x, 350.),
            );
        }
    }
}

fn drop_piece(
    mut cmd: Commands,
    mouse_input: Res<Input<MouseButton>>,
    dragged_query: Query<(Entity, &Piece, &TileCoords), With<Dragged>>,
) {
    if mouse_input.just_released(MouseButton::Left) {
        for (dragged_e, ..) in dragged_query.iter() {
            cmd.entity(dragged_e).remove::<Dragged>();
        }
    }
}

fn drag_piece(
    mut cmd: Commands,
    mouse_input: Res<Input<MouseButton>>,
    board: Res<Board>,
    pieces: Res<Pieces>,
    dragged_query: Query<(Entity, &Piece, &TileCoords), (With<Dragged>, Changed<TileCoords>)>,
) {
    if mouse_input.pressed(MouseButton::Left) {
        if let Ok((_, piece, coords)) = dragged_query.get_single() {
            if let Some(coords) = coords.tile_coords {
                if let Ok(_) = board.can_place_piece(
                    coords.x as usize,
                    coords.y as usize,
                    pieces.pieces[piece.0].get_fields(),
                ) {
                    info!("can place!");
                }
            }
        }
    }
}

fn limit_drag_count(mut cmd: Commands, dragged_query: Query<Entity, With<Dragged>>) {
    if dragged_query.iter().len() > 1 {
        for e in dragged_query.iter().skip(1) {
            cmd.entity(e).remove::<Dragged>();
        }
    }
}

fn process_movers(
    mover_q: Query<(&Mover, &Transform, &TileCoords, &Interactable)>,
    mut moved_q: Query<&mut Transform, Without<Mover>>,
) {
    for (mover, mover_t, coords, interactable) in mover_q.iter() {
        if let Ok(mut t) = moved_q.get_mut(mover.moved_e) {
            t.translation = if let Some(pos) = coords.tile_coords {
                (get_world_coords_from_tile(pos)
                    + Vec2::new(-BOARD_SIZE_PX / 2., BOARD_SIZE_PX / 2.)
                    + Vec2::new(
                        interactable.bounding_box.0.x.abs(),
                        -interactable.bounding_box.0.y.abs(),
                    ))
                .extend(0.)
            } else {
                mover_t.translation
            };
        }
    }
}
