use std::ops::{Add, Div};

use crate::{board::Board, mouse::CursorWorldPosition, GameState};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use iyes_loopless::prelude::*;

pub struct TilePlacementPlugin;
impl Plugin for TilePlacementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileCoords>()
            .insert_resource(Blocks {
                blocks: vec![
                    vec![0, 1],
                    vec![0, 9],
                    vec![0, 1, 2],
                    vec![0, 9, 18],
                    vec![0, 1, 9, 10],
                    vec![0, 1, 2, 9],
                ],
            })
            .add_enter_system(GameState::Playing, setup_board)
            .add_system(update_tile_coords)
            .add_system(place_square_on_click.run_if_resource_exists::<Board>());
        //.add_system(log_tile_coords);
    }
}

#[derive(Default, Debug)]
pub struct TileCoords(pub UVec2);

pub struct Blocks {
    blocks: Vec<Vec<usize>>,
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
    ));

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
    ));

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
    ));

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

fn place_square_on_click(mut board: ResMut<Board>, tile_coords: Res<TileCoords>) {
    if tile_coords.is_changed() {
        let piece = [0, 1, 9, 10];
        if let Ok(_) =
            board.can_place_piece(tile_coords.0.x as usize, tile_coords.0.y as usize, &piece)
        {
            let place_res =
                board.place_piece(tile_coords.0.x as usize, tile_coords.0.y as usize, &piece);

            info!("Place res: {:?}", place_res);
        }
    }
}
