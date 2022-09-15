#![allow(dead_code)]
use bevy::prelude::*;
use std::{
    iter,
    ops::{Div, Mul, Range, Rem},
};

#[derive(Component)]
pub struct Piece;

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

        if fields.len() == 0 {
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
