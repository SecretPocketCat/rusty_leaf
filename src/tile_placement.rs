use crate::{
    board::Board,
    mouse::CursorWorldPosition,
    piece::{Piece, PieceFields},
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

pub const BOARD_SIZE: usize = 9;
pub const SECTION_SIZE: usize = 3;

pub struct TilePlacementPlugin;
impl Plugin for TilePlacementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileCoords>()
            // .add_plugin(InspectorPlugin::<Board>::new())
            .insert_resource(Pieces {
                pieces: vec![
                    PieceFields::new(&[0, 1], 2, BOARD_SIZE),
                    PieceFields::new(&[0, 1], 1, BOARD_SIZE),
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
            .add_enter_system(GameState::Playing, setup_board)
            .add_system(fill_piece_queue.run_if_resource_exists::<Pieces>())
            .add_system(drag_piece)
            .add_system(log_tile_coords)
            .add_system(update_tile_coords);
    }
}

#[derive(Default, Debug)]
pub struct TileCoords {
    cursor_tile_coords: UVec2,
    dragged_item_tile_coords: Option<UVec2>,
}

pub struct Pieces {
    pieces: Vec<PieceFields>,
    queue: VecDeque<usize>,
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

fn update_tile_coords(
    cursor_pos: Res<CursorWorldPosition>,
    mut tile_coords: ResMut<TileCoords>,
    dragged_query: Query<(&Transform, &Interactable), With<Dragged>>,
) {
    // todo: use grabbed tile coords (top left)
    if cursor_pos.is_changed() {
        let cursor_coords = get_tile_coords_from_world(cursor_pos.0);
        if tile_coords.cursor_tile_coords != cursor_coords {
            tile_coords.cursor_tile_coords = cursor_coords;
        }

        if let Ok((interactable_t, interactable)) = dragged_query.get_single() {
            let dragged_tile_coords = get_tile_coords_from_world(
                interactable_t.translation.truncate()
                    + Vec2::new(15., interactable.bounding_box.0.y),
            );

            match tile_coords.dragged_item_tile_coords {
                Some(coords) => {
                    if coords != dragged_tile_coords {
                        tile_coords.dragged_item_tile_coords = Some(cursor_coords);
                    }
                }
                None => tile_coords.dragged_item_tile_coords = Some(dragged_tile_coords),
            }
        } else if tile_coords.dragged_item_tile_coords.is_some() {
            tile_coords.dragged_item_tile_coords = None;
        }
    }
}

fn get_tile_coords_from_world(world_coords: Vec2) -> UVec2 {
    let coords = world_coords.div(60.).round().add(Vec2::splat(4.));
    UVec2::new(coords.x as u32, 8u32.saturating_sub(coords.y.abs() as u32))
}

fn log_tile_coords(cursor_pos: Res<CursorWorldPosition>, tile_coords: Res<TileCoords>) {
    if tile_coords.is_changed() {
        info!(
            "Tile coords [{}, {}]; Dragged tile coords [{:?}]; Cursor coords [{}, {}]",
            tile_coords.cursor_tile_coords.x,
            tile_coords.cursor_tile_coords.y,
            tile_coords.dragged_item_tile_coords,
            cursor_pos.0.x,
            cursor_pos.0.y
        );
    }
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
                Vec2::new(x, 350.),
            );
        }
    }
}

fn spawn_piece(cmd: &mut Commands, piece: &PieceFields, position: Vec2) {
    let size = 60.;
    let size_h = size / 2.;
    let corner = Vec2::new(
        piece.get_width() as f32 * size_h,
        piece.get_height() as f32 * size_h,
    );

    info!("piece height: {}", piece.get_height());

    cmd.spawn_bundle(SpatialBundle {
        transform: Transform::from_xyz(position.x, position.y, 1.),
        ..default()
    })
    .with_children(|b| {
        let piece_padded_w = piece.get_padded_width();
        let piece_offset_x = piece.get_width().sub(1) as f32 / 2.;
        let piece_offset_y = piece.get_height().sub(1) as f32 / 2.;
        for i in piece.get_fields().iter() {
            let x = ((i % piece_padded_w) as f32 - piece_offset_x) * size;
            let y = ((i / piece_padded_w) as f32 - piece_offset_y) * -size;

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
    .insert(Interactable {
        groups: vec![bevy_interact_2d::Group(0)],
        bounding_box: (-corner, corner),
        ..default()
    })
    .insert(Draggable {
        groups: vec![bevy_interact_2d::Group(0)],
        // hook: Some(Vec2::new(0., 60.)),
        ..Default::default()
    })
    .insert(Piece)
    .insert(Name::new("piece"));
}

fn drag_piece(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    dragged_query: Query<Entity, (With<Dragged>, With<Piece>)>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        for dragged in dragged_query.iter() {
            commands.entity(dragged).remove::<Dragged>();
        }
    }
}
