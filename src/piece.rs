#![allow(dead_code)]
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::Animator;

use std::ops::{Div, Sub};

use crate::{
    coords::TileCoords,
    drag::Draggable,
    interaction::{Interactable, InteractionGroup},
    mover::Mover,
    render::{ZIndex, COL_DARK},
    tile_placement::TILE_SIZE,
    tween::{delay_tween, get_relative_move_tween},
};

#[derive(Component)]
pub struct Piece(pub usize);

#[derive(Component)]
pub struct FieldCoords(pub UVec2);

#[derive(Component, Inspectable)]
pub struct PlacedFieldIndex(pub usize);

#[derive(Clone)]
pub struct PieceFields {
    width: usize,
    padded_width: usize,
    height: usize,
    fields: Vec<usize>,
}

impl PieceFields {
    pub fn new(fields: &[usize], width: usize, padded_width: usize) -> Self {
        if width > padded_width {
            panic!("Piece is too wide {width} for the padded width {padded_width}");
        }

        if fields.is_empty() {
            panic!("No fields");
        }

        let fields: Vec<usize> = if width == padded_width {
            fields.into()
        } else {
            fields
                .iter()
                .map(|f| f + f.div(width) * (padded_width - width))
                .collect()
        };

        let height = *fields.iter().max().unwrap() / padded_width + 1;

        Self {
            width,
            padded_width,
            height,
            fields,
        }
    }

    pub fn get_fields(&self) -> &[usize] {
        &self.fields
    }

    pub fn get_padded_width(&self) -> usize {
        self.padded_width
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }
}

pub fn spawn_piece(
    cmd: &mut Commands,
    piece: &PieceFields,
    piece_index: usize,
    position: Vec2,
    tween_delay: u64,
) {
    let size_h = TILE_SIZE / 2.;
    let corner = Vec2::new(
        piece.get_width() as f32 * size_h,
        piece.get_height() as f32 * size_h,
    );

    let piece_visual_e = cmd
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_xyz(position.x, position.y, 1.),
            ..default()
        })
        .insert(ZIndex::Piece)
        .with_children(|b| {
            let piece_padded_w = piece.get_padded_width();
            let piece_offset_x = piece.get_width().sub(1) as f32 / 2.;
            let piece_offset_y = piece.get_height().sub(1) as f32 / 2.;
            for i in piece.get_fields().iter() {
                let x = i % piece_padded_w;
                let y = i / piece_padded_w;

                let pos = Vec3::new(
                    (x as f32 - piece_offset_x) * TILE_SIZE,
                    (y as f32 - piece_offset_y) * -TILE_SIZE,
                    0.,
                );
                b.spawn_bundle(GeometryBuilder::build_as(
                    &shapes::Rectangle {
                        extents: Vec2::splat(TILE_SIZE),
                        ..default()
                    },
                    DrawMode::Outlined {
                        outline_mode: StrokeMode::new(COL_DARK, 4.),
                        fill_mode: FillMode::color(Color::rgb_u8(254, 209, 113)),
                    },
                    Transform::from_translation(Vec3::new(0., 63., pos.z)),
                ))
                .insert(Animator::new(delay_tween(
                    get_relative_move_tween(pos, 450, None),
                    tween_delay,
                )))
                .insert(FieldCoords(UVec2::new(x as u32, y as u32)))
                .insert(Name::new("field"));
            }
        })
        .insert(Name::new("piece_visual"))
        .id();

    cmd.spawn_bundle(SpatialBundle {
        transform: Transform::from_xyz(position.x, position.y, 1.),
        ..default()
    })
    .insert(Interactable::new_rectangle(InteractionGroup::Piece, corner))
    .insert(Draggable)
    .insert(Piece(piece_index))
    .insert(TileCoords::default())
    .insert(Mover {
        moved_e: piece_visual_e,
    })
    .insert(Name::new("piece"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(vec![0, 1, 2, 5, 6], 3, 2 => panics)]
    #[test_case(vec![0, 1, 2, 5, 6], 3, 3 => vec![0, 1, 2, 5, 6])]
    #[test_case(vec![0, 1, 2, 5, 6], 3, 5 => vec![0, 1, 2, 7, 10])]
    #[test_case(vec![0, 1, 2, 3], 2, 3 => vec![0, 1, 3, 4])]
    fn new(fields: Vec<usize>, width: usize, padded_width: usize) -> Vec<usize> {
        let piece = PieceFields::new(&fields, width, padded_width);
        piece.fields
    }
}
