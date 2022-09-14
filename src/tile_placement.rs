use crate::{board::Board, mouse::CursorWorldPosition, piece::Piece, GameState};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use iyes_loopless::prelude::*;
use rand::Rng;
use std::ops::{Add, Div};

const BOARD_SIZE: usize = 9;

pub struct TilePlacementPlugin;
impl Plugin for TilePlacementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileCoords>()
            .insert_resource(Pieces {
                pieces: vec![
                    Piece::new(&[0, 1], 2, BOARD_SIZE),
                    Piece::new(&[0, 1], 1, BOARD_SIZE),
                    Piece::new(&[0, 1, 2, 3], 2, BOARD_SIZE),
                    Piece::new(&[0, 1, 2], 2, BOARD_SIZE),
                    Piece::new(&[0, 1, 3], 2, BOARD_SIZE),
                    Piece::new(&[0, 1, 2, 5], 3, BOARD_SIZE),
                    Piece::new(&[0, 1, 2, 3], 3, BOARD_SIZE),
                    Piece::new(&[0, 1, 2, 4], 3, BOARD_SIZE),
                    Piece::new(&[1, 3, 4, 5], 3, BOARD_SIZE),
                ],
            })
            .add_enter_system(GameState::Playing, setup_board)
            .add_system(spawn_piece.run_if_resource_exists::<Board>())
            .add_system(update_tile_coords);
        //.add_system(log_tile_coords);
    }
}

#[derive(Default, Debug)]
pub struct TileCoords(pub UVec2);

pub struct Pieces {
    pieces: Vec<Piece>,
}

fn setup_board(mut cmd: Commands) {
    let size = 540.;
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

fn update_tile_coords(cursor_pos: Res<CursorWorldPosition>, mut tile_coords: ResMut<TileCoords>) {
    if cursor_pos.is_changed() {
        let coords = cursor_pos.0.div(60.).round().add(Vec2::splat(4.));
        let coords = UVec2::new(coords.x as u32, 8u32.saturating_sub(coords.y.abs() as u32));

        if tile_coords.0 != coords {
            tile_coords.0 = coords;
        }
    }
}

fn log_tile_coords(cursor_pos: Res<CursorWorldPosition>, tile_coords: Res<TileCoords>) {
    if tile_coords.is_changed() {
        info!(
            "Tile coords [{}, {}]; Cursor coords [{}, {}]",
            tile_coords.0.x as usize, tile_coords.0.y as usize, cursor_pos.0.x, cursor_pos.0.y
        );
    }
}

fn spawn_piece(
    mut cmd: Commands,
    mut board: ResMut<Board>,
    pieces: Res<Pieces>,
    buttons: Res<Input<MouseButton>>,
    tile_coords: Res<TileCoords>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let piece = pieces
            .pieces
            .get(2)
            // .get(rand::thread_rng().gen_range(0..pieces.pieces.len()))
            .unwrap();

        match board.can_place_piece(
            tile_coords.0.x as usize,
            tile_coords.0.y as usize,
            &piece.get_fields(),
        ) {
            Ok(_) => {
                let place_res = board.place_piece(
                    tile_coords.0.x as usize,
                    tile_coords.0.y as usize,
                    &&piece.get_fields(),
                );

                info!("Place res: {:?}", place_res);

                cmd.spawn_bundle(TransformBundle {
                    local: Transform::from_xyz(0., 0., 100.),
                    ..default()
                })
                .insert_bundle(VisibilityBundle::default())
                .with_children(|b| {
                    let size = 60.;
                    let piece_w = piece.get_width();
                    for i in piece.get_fields().iter() {
                        let x = (tile_coords.0.x as f32 - 4.) * size + (i % piece_w) as f32 * size;
                        let y = (3. - tile_coords.0.y as f32) * size + (i / piece_w) as f32 * size;

                        b.spawn_bundle(GeometryBuilder::build_as(
                            &shapes::Rectangle {
                                extents: Vec2::splat(size),
                                ..default()
                            },
                            DrawMode::Fill(FillMode::color(Color::ORANGE)),
                            Transform::from_xyz(x, y, 0.),
                        ));
                    }
                })
                .insert(Name::new("piece"));
            }
            Err(e) => {
                warn!("Failed to place a piece {:?}", e);
            }
        }
    }
}
