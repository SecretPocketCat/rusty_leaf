#![allow(dead_code)]
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use std::{
    collections::VecDeque,
    iter,
    ops::{Div, Mul, Range, Rem},
};

use crate::tile_placement::{BOARD_SIZE, SECTION_SIZE};

#[derive(Debug, Inspectable)]
pub struct Board {
    width: usize,
    heigth: usize,
    section_size: usize,
    fields: Vec<bool>,
}

impl Default for Board {
    fn default() -> Self {
        Self::new(BOARD_SIZE, BOARD_SIZE, SECTION_SIZE)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BoardClear {
    Row(usize),
    Column(usize),
    Section {
        section_index: usize,
        used_special: bool,
    },
}

#[derive(Debug, Default)]
pub struct BoardClearQueue {
    pub queue: VecDeque<BoardClear>,
}

#[derive(Debug)]
pub enum PlaceError {
    Taken,
    OutOfBounds,
}

impl Board {
    pub fn new(width: usize, heigth: usize, section_size: usize) -> Self {
        Self::with_fields(
            width,
            heigth,
            section_size,
            iter::repeat(false).take(width * heigth).collect(),
        )
    }

    fn with_fields(width: usize, heigth: usize, section_size: usize, fields: Vec<bool>) -> Self {
        if width == 0 || heigth == 0 || section_size == 0 {
            panic!("Invalid dimension - no dimension can be 0");
        }

        if width * heigth % (section_size * section_size) != 0 {
            panic!("Invalid section size {section_size}");
        }

        if fields.len() != width * heigth {
            panic!(
                "Invalid fields len {}, should be {}",
                fields.len(),
                width * heigth
            );
        }

        Self {
            width,
            heigth,
            section_size,
            fields,
        }
    }

    pub fn can_place_piece(&self, x: usize, y: usize, piece: &[usize]) -> Result<(), PlaceError> {
        let mut res = Ok(());

        for field in piece {
            let i = field + x + y * self.width;
            let offset_x = field.rem_euclid(self.width) + x;
            if i >= self.width * self.heigth || offset_x >= self.width || y >= self.heigth {
                // out of bounds has higher prio
                return Err(PlaceError::OutOfBounds);
            } else if self.fields[i] {
                res = Err(PlaceError::Taken);
            }
        }

        res
    }

    pub fn place_piece(
        &mut self,
        x: usize,
        y: usize,
        piece: &[usize],
    ) -> Result<Vec<BoardClear>, PlaceError> {
        self.can_place_piece(x, y, piece)?;

        let mut cleared = Vec::new();

        for field in piece {
            let x = (field + x).rem(self.width);
            let y = field / self.width + y;

            let col_done = self.column_done(x);
            let row_done = self.row_done(y);
            let section_done = self.section_done(x, y);

            self.fields[y * self.width + x] = true;

            if !col_done && self.column_done(x) {
                cleared.push(BoardClear::Column(x));
            }

            if !row_done && self.row_done(y) {
                cleared.push(BoardClear::Row(y));
            }

            if !section_done.1 && self.section_done(x, y).1 {
                cleared.push(BoardClear::Section {
                    section_index: section_done.0,
                    used_special: false,
                });
            }
        }

        Ok(cleared)
    }

    fn get_column(&self, col: usize) -> Vec<usize> {
        if col >= self.width {
            panic!("Column [{col}] is out of bounds");
        }

        (col..self.width * self.heigth)
            .into_iter()
            .filter(|i| *i % self.width == col)
            .collect()
    }

    fn get_row_range(&self, row: usize) -> Range<usize> {
        if row >= self.heigth {
            panic!("Row [{row}] is out of bounds");
        }

        let from = row * self.width;
        from..from + self.width
    }

    fn get_section(&self, x: usize, y: usize) -> (usize, Vec<usize>) {
        if x >= self.width || y >= self.heigth {
            panic!("Section coords [{x}, {y}] are out of bounds");
        }

        let mut section = Vec::with_capacity(self.section_size * self.section_size);

        let row = y.div_floor(self.section_size);
        let col = x.div_floor(self.section_size);

        let moved_y = row * self.section_size;
        for y in moved_y..moved_y + self.section_size {
            for x in 0..self.section_size {
                section.push(y * self.width + col * self.section_size + x);
            }
        }

        ((self.width / self.section_size) * row + col, section)
    }

    fn get_section_coords_from_section_index(&self, index: usize) -> (usize, usize) {
        let x = index.rem_euclid(self.width / self.section_size) * self.section_size;
        let y = index
            .div(self.width / self.section_size)
            .mul(self.section_size);

        (x, y)
    }

    fn get_section_by_section_index(&self, index: usize) -> Vec<usize> {
        let coords = self.get_section_coords_from_section_index(index);
        self.get_section(coords.0, coords.1).1
    }

    fn section_done(&self, x: usize, y: usize) -> (usize, bool) {
        let section = self.get_section(x, y);
        (section.0, section.1.iter().all(|i| self.fields[*i]))
    }

    fn row_done(&self, row: usize) -> bool {
        self.get_row_range(row).all(|i| self.fields[i])
    }

    fn column_done(&self, column: usize) -> bool {
        self.get_column(column).iter().all(|i| self.fields[*i])
    }

    pub fn clear_column(&mut self, column: usize) -> Vec<usize> {
        let col = self.get_column(column);
        for i in col.iter() {
            self.fields[*i] = false;
        }

        col
    }

    pub fn clear_row(&mut self, row: usize) -> Vec<usize> {
        let mut row_i = Vec::new();
        for i in self.get_row_range(row) {
            self.fields[i] = false;
            row_i.push(i);
        }

        row_i
    }

    pub fn clear_section(&mut self, section: usize) -> Vec<usize> {
        let section = self.get_section_by_section_index(section);
        for i in section.iter() {
            self.fields[*i] = false;
        }

        section
    }

    pub fn clear(&mut self) {
        for f in self.fields.iter_mut() {
            *f = false;
        }
    }

    pub fn tile_coords_to_tile_index(&self, coords: UVec2) -> usize {
        (coords.y * self.width as u32 + coords.x) as usize
    }

    pub fn is_section_empty(&self, section_index: usize) -> bool {
        self.get_section_by_section_index(section_index)
            .iter()
            .all(|i| !self.fields[*i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;
    use test_case::test_case;

    #[test_case(1, 1, 1)]
    #[test_case(3, 3, 1)]
    #[test_case(3, 3, 3)]
    #[test_case(12, 9, 3)]
    #[test_case(10, 10, 3 => panics)]
    #[test_case(0, 1, 1 => panics)]
    #[test_case(1, 0, 1 => panics)]
    #[test_case(1, 1, 0 => panics)]
    fn new(width: usize, heigth: usize, section_size: usize) {
        Board::new(width, heigth, section_size);
    }

    #[test_case(2, 1, 1, vec![true, false])]
    #[test_case(2, 1, 1, vec![true] => panics)]
    fn new_with_fields(width: usize, heigth: usize, section_size: usize, fields: Vec<bool>) {
        Board::with_fields(width, heigth, section_size, fields);
    }

    #[test_case(0, 0 => matches Ok(_))]
    #[test_case(3, 3 => matches Ok(_))]
    #[test_case(4, 6 => matches Ok(_))]
    #[test_case(5, 0 => matches Err(PlaceError::OutOfBounds))]
    #[test_case(50, 0 => matches Err(PlaceError::OutOfBounds))]
    #[test_case(0, 7 => matches Err(PlaceError::OutOfBounds))]
    #[test_case(0, 50 => matches Err(PlaceError::OutOfBounds))]
    fn can_place_l_piece(x: usize, y: usize) -> Result<(), PlaceError> {
        let board = Board::new(6, 9, 3);
        // L piece
        board.can_place_piece(x, y, &[0, 6, 12, 13])
    }

    #[test_case(0, 0 => matches Err(PlaceError::Taken))]
    #[test_case(1, 0 => matches Err(PlaceError::Taken))]
    #[test_case(0, 1 => matches Err(PlaceError::Taken))]
    #[test_case(2, 2 => matches Ok(_))]
    #[test_case(4, 4 => matches Ok(_))]
    fn can_place_square_piece_maybe_taken(x: usize, y: usize) -> Result<(), PlaceError> {
        // square piece
        let piece = [0, 1, 6, 7];
        let mut fields = [false; 54];
        for i in piece {
            fields[i] = true;
        }
        let board = Board::with_fields(6, 9, 3, fields.into());

        board.can_place_piece(x, y, &piece)
    }

    #[test_case(0 => vec![0, 4])]
    #[test_case(1 => vec![1, 5])]
    #[test_case(2 => vec![2, 6])]
    #[test_case(5 => panics)]
    fn get_column(column: usize) -> Vec<usize> {
        let board = Board::new(4, 2, 2);
        board.get_column(column)
    }

    #[test_case(0 => vec![0, 3, 6])]
    #[test_case(1 => vec![1, 4, 7])]
    #[test_case(2 => vec![2, 5, 8])]
    #[test_case(4 => panics)]
    fn get_column_3_x_3(column: usize) -> Vec<usize> {
        let board = Board::new(3, 3, 3);
        board.get_column(column)
    }

    #[test_case(0 => 0..4)]
    #[test_case(2 => 8..12)]
    #[test_case(5 => panics)]
    fn get_row_range(row: usize) -> Range<usize> {
        let board = Board::new(4, 3, 2);
        board.get_row_range(row)
    }

    #[test_case(0 => 0..3)]
    #[test_case(1 => 3..6)]
    #[test_case(2 => 6..9)]
    #[test_case(4 => panics)]
    fn get_row_range_3_x_3(row: usize) -> Range<usize> {
        let board = Board::new(3, 3, 3);
        board.get_row_range(row)
    }

    #[test_case(0, 0 => (0, vec![0, 1, 4, 5]))]
    #[test_case(1, 1 => (0, vec![0, 1, 4, 5]))]
    #[test_case(2, 0 => (1, vec![2, 3, 6, 7]))]
    #[test_case(0, 2 => (2, vec![8, 9, 12, 13]))]
    #[test_case(2, 2 => (3, vec![10, 11, 14, 15]))]
    fn get_section(x: usize, y: usize) -> (usize, Vec<usize>) {
        let board = Board::new(4, 4, 2);

        board.get_section(x, y)
    }

    #[test_case(0, 6, 4, 2 => (0, 0))]
    #[test_case(2, 6, 4, 2 => (4, 0))]
    #[test_case(5, 6, 4, 2 => (4, 2))]
    #[test_case(6, 6, 4, 2 => (0, 4))]
    #[test_case(2, 6, 2, 2 => (4, 0))]
    fn get_section_coords_from_section_index(
        index: usize,
        width: usize,
        height: usize,
        section_size: usize,
    ) -> (usize, usize) {
        let board = Board::new(width, height, section_size);

        board.get_section_coords_from_section_index(index)
    }

    #[test_case(0 => vec![0, 1, 4, 5])]
    #[test_case(1 => vec![2, 3, 6, 7])]
    #[test_case(2 => vec![8, 9, 12, 13])]
    #[test_case(3 => vec![10, 11, 14, 15])]
    fn get_section_by_section_index(section_index: usize) -> Vec<usize> {
        let board = Board::new(4, 4, 2);

        board.get_section_by_section_index(section_index)
    }

    #[test_case(0 => vec![0, 1, 6, 7])]
    #[test_case(1 => vec![2, 3, 8, 9])]
    #[test_case(2 => vec![4, 5, 10, 11])]
    fn get_section_by_section_index_wide_board(section_index: usize) -> Vec<usize> {
        let board = Board::new(6, 2, 2);

        board.get_section_by_section_index(section_index)
    }

    #[test_case(0 => true)]
    #[test_case(1 => false)]
    fn row_done(row: usize) -> bool {
        let board = Board::with_fields(2, 2, 2, [true, true, true, false].into());

        board.row_done(row)
    }

    #[test_case(0 => true)]
    #[test_case(1 => false)]
    fn column_done(col: usize) -> bool {
        let board = Board::with_fields(2, 2, 2, [true, true, true, false].into());

        board.column_done(col)
    }

    #[test_case(0, 0 => (0, true))]
    #[test_case(1, 1 => (0, true))]
    #[test_case(0, 2 => (1, false))]
    fn section_done(x: usize, y: usize) -> (usize, bool) {
        let board = Board::with_fields(
            2,
            4,
            2,
            [true, true, true, true, true, true, false, false].into(),
        );

        board.section_done(x, y)
    }

    #[test_case(0 => vec![false, true, false, true])]
    #[test_case(1 => vec![true, false, true, false])]
    fn clear_column(col: usize) -> Vec<bool> {
        let mut board = Board::with_fields(2, 2, 2, [true; 4].into());

        board.clear_column(col);
        board.fields
    }

    #[test_case(0 => vec![false, false, true, true])]
    #[test_case(1 => vec![true, true, false, false])]
    fn clear_row(row: usize) -> Vec<bool> {
        let mut board = Board::with_fields(2, 2, 2, [true; 4].into());

        board.clear_row(row);
        board.fields
    }

    #[test_case(0 => vec![false, false, true, true, false, false, true, true])]
    #[test_case(1 => vec![true, true, false, false, true, true, false, false])]
    fn clear_section(section: usize) -> Vec<bool> {
        let mut board = Board::with_fields(4, 2, 2, [true; 8].into());

        board.clear_section(section);
        board.fields
    }

    #[test_case(Vec::<usize>::new() => Vec::<BoardClear>::new())]
    #[test_case(vec![6, 7, 10, 11, 15] => Vec::<BoardClear>::new())]
    #[test_case(vec![2, 3] => vec![BoardClear::Row(0)])]
    #[test_case(vec![8, 12] => vec![BoardClear::Column(0)])]
    #[test_case(vec![5] => vec![BoardClear::Section { section_index: 0, used_special: false }])]
    #[test_case(vec![2, 3, 5, 8, 12] => vec![BoardClear::Row(0), BoardClear::Column(0), BoardClear::Section { section_index: 0, used_special: false }])]
    fn place_piece(taken: Vec<usize>) -> Vec<BoardClear> {
        let mut fields = [false; 16];

        for i in taken {
            fields[i] = true;
        }

        let mut board = Board::with_fields(4, 4, 2, fields.into());
        // corner piece
        board.place_piece(0, 0, &[0, 1, 4]).unwrap()
    }

    #[test]
    fn place_piece_cross() {
        let mut fields = [false; 9];
        for i in [0, 2, 6, 8] {
            fields[i] = true;
        }

        let mut board = Board::with_fields(3, 3, 3, fields.into());
        // cross piece
        let res = board.place_piece(0, 0, &[1, 3, 4, 5, 7]).unwrap();

        assert_eq!(Vec::<bool>::from([true; 9]), board.fields);

        assert_that(&res.iter()).contains_all_of(
            &vec![
                BoardClear::Row(0),
                BoardClear::Row(1),
                BoardClear::Row(2),
                BoardClear::Column(0),
                BoardClear::Column(1),
                BoardClear::Column(2),
                BoardClear::Section {
                    section_index: 0,
                    used_special: false,
                },
            ]
            .iter(),
        );
    }

    #[test]
    fn place_piece_double_corner() {
        let mut fields = [false; 9];
        for i in [2, 5, 6, 7] {
            fields[i] = true;
        }

        let mut board = Board::with_fields(3, 3, 3, fields.into());
        // square piece
        let res = board.place_piece(0, 0, &[0, 1, 3, 4]).unwrap();

        let mut expected_flds = Vec::from([true; 9]);
        expected_flds[8] = false;

        assert_eq!(expected_flds, board.fields);

        assert_that(&res.iter()).contains_all_of(
            &vec![
                BoardClear::Row(0),
                BoardClear::Row(1),
                BoardClear::Column(0),
                BoardClear::Column(1),
            ]
            .iter(),
        );
    }
}
