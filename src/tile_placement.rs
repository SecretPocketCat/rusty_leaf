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
            .add_system(drag_piece)
            .add_system(log_tile_coords)
            .add_system(update_tile_coords);
    }
}

#[derive(Default, Debug)]
pub struct TileCoords {
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
    if cursor_pos.is_changed() {
        if let Ok((interactable_t, interactable)) = dragged_query.get_single() {
            let dragged_tile_coords = get_tile_coords_from_world(
                interactable_t.translation.truncate()
                    + Vec2::new(
                        -interactable.bounding_box.0.x.abs() + 90.,
                        interactable.bounding_box.0.y.abs() - 30.,
                    ),
            );

            if tile_coords.dragged_item_tile_coords.is_some() && dragged_tile_coords.is_none() {
                tile_coords.dragged_item_tile_coords = dragged_tile_coords;
            } else {
                match tile_coords.dragged_item_tile_coords {
                    Some(prev_coords) => match dragged_tile_coords {
                        Some(new_coords) => {
                            if prev_coords != new_coords {
                                tile_coords.dragged_item_tile_coords = dragged_tile_coords;
                            }
                        }
                        None => tile_coords.dragged_item_tile_coords = dragged_tile_coords,
                    },
                    None => tile_coords.dragged_item_tile_coords = dragged_tile_coords,
                }
            }
        } else if tile_coords.dragged_item_tile_coords.is_some() {
            tile_coords.dragged_item_tile_coords = None;
        }
    }
}

fn get_tile_coords_from_world(world_coords: Vec2) -> Option<UVec2> {
    let max_i = BOARD_SIZE as f32 - 1.;
    let base_coords = world_coords.div(60.).round().add(Vec2::splat(4.));
    let coords = Vec2::new(base_coords.x - 1., max_i - base_coords.y.abs());

    if coords.x >= 0. && coords.x <= max_i && coords.y >= 0. && coords.y <= max_i {
        Some(UVec2::new(coords.x as u32, coords.y as u32))
    } else {
        None
    }
}

fn log_tile_coords(cursor_pos: Res<CursorWorldPosition>, tile_coords: Res<TileCoords>) {
    if tile_coords.is_changed() {
        info!(
            "Dragged tile coords [{:?}]; Cursor coords [{}, {}]",
            tile_coords.dragged_item_tile_coords, cursor_pos.0.x, cursor_pos.0.y
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
                *piece_i,
                Vec2::new(x, 350.),
            );
        }
    }
}

fn spawn_piece(cmd: &mut Commands, piece: &PieceFields, piece_index: usize, position: Vec2) {
    let size = 60.;
    let size_h = size / 2.;
    let corner = Vec2::new(
        piece.get_width() as f32 * size_h,
        piece.get_height() as f32 * size_h,
    );

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
    .insert(Piece(piece_index))
    .insert(Name::new("piece"));
}

fn drag_piece(
    mut cmd: Commands,
    mouse_input: Res<Input<MouseButton>>,
    tile_coords: Res<TileCoords>,
    board: Res<Board>,
    pieces: Res<Pieces>,
    dragged_query: Query<(Entity, &Piece), With<Dragged>>,
) {
    if mouse_input.just_released(MouseButton::Left) {
        for (dragged_e, _) in dragged_query.iter() {
            cmd.entity(dragged_e).remove::<Dragged>();
        }
    } else if mouse_input.pressed(MouseButton::Left) {
        if tile_coords.is_changed() {
            if dragged_query.iter().len() > 0 {
                if let Some(coords) = tile_coords.dragged_item_tile_coords {
                    if let Ok((_, piece)) = dragged_query.get_single() {
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
    }
}

fn limit_drag_count(mut cmd: Commands, dragged_query: Query<Entity, With<Dragged>>) {
    if dragged_query.iter().len() > 1 {
        for e in dragged_query.iter().skip(1) {
            cmd.entity(e).remove::<Dragged>();
        }
    }
}
