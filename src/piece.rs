#![allow(dead_code)]
use bevy::prelude::*;
use std::{
    iter,
    ops::{Div, Mul, Range, Rem},
};

pub struct Piece {
    width: usize,
    fields: Vec<usize>,
}

impl Piece {
    pub fn new(fields: &[usize], width: usize, padded_width: usize) -> Self {
        if width > padded_width {
            panic!("Piece is too wide {width} for the padded width {padded_width}");
        }

        let fields: Vec<usize> = if width == padded_width {
            fields.into()
        } else {
            fields
                .iter()
                .map(|f| f + f.div(width) * (padded_width - width))
                .collect()
        };

        Self { width, fields }
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
        let piece = Piece::new(&fields, width, padded_width);
        piece.fields
    }
}
