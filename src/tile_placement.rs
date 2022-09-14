use crate::{mouse::CursorWorldPosition, GameState};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{tess::geom::euclid::num::Floor, *};
use iyes_loopless::prelude::*;

pub struct TilePlacementPlugin;

impl Plugin for TilePlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Playing, setup_board)
            .add_system(update_tile_coords);
    }
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
}

fn update_tile_coords(cursor_pos: Res<CursorWorldPosition>) {
    // info!("{}", cursor_pos.0);
}
