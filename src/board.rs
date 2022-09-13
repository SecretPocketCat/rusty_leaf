#![allow(dead_code)]
use std::iter;

struct Board {
    width: usize,
    heigth: usize,
    section_size: usize,
    fields: Vec<bool>,
}

enum BoardClear {
    Row(usize),
    Column(usize),
    Section(usize),
}

#[derive(Debug)]
enum PlaceError {
    Taken,
    OutOfBounds,
}

impl Board {
    pub fn new(width: usize, heigth: usize, section_size: usize) -> Self {
        Self::new_with_fields(
            width,
            heigth,
            section_size,
            iter::repeat(false).take(width * heigth).collect(),
        )
    }

    fn new_with_fields(
        width: usize,
        heigth: usize,
        section_size: usize,
        fields: Vec<bool>,
    ) -> Self {
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

        return res;
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
            let col_done = self.column_done(x);
            let row_done = self.row_done(y);
            let section_done = self.section_done(x, y);

            self.fields[field + x + y * self.width] = true;

            if !col_done && self.column_done(x) {
                cleared.push(BoardClear::Column(x));
            }

            if !row_done && self.row_done(y) {
                cleared.push(BoardClear::Row(y));
            }

            if !section_done.1 && self.section_done(x, y).1 {
                cleared.push(BoardClear::Section(section_done.0));
            }
        }

        for x in cleared.iter() {
            match x {
                BoardClear::Row(row) => self.clear_row(*row),
                BoardClear::Column(col) => self.clear_column(*col),
                BoardClear::Section(section) => self.clear_section(*section),
            };
        }

        Ok(cleared)
    }

    fn row_done(&self, row: usize) -> bool {
        let from = row * self.width;
        (from..from + self.width).all(|i| self.fields[i])
    }

    fn column_done(&self, column: usize) -> bool {
        self.fields
            .iter()
            .enumerate()
            .filter(|(i, _)| i % (self.width + column) == 0)
            .all(|(_, taken)| *taken)
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

    fn get_section_by_section_index(&self, index: usize) -> Vec<usize> {
        let x = index.rem_euclid(self.width / self.section_size) * self.section_size;
        let y = index.div_floor(self.heigth / self.section_size) * self.section_size;
        println!(" index {index} => {x}, {y}");

        self.get_section(x, y).1
    }

    fn section_done(&self, x: usize, y: usize) -> (usize, bool) {
        let section = self.get_section(x, y);
        (section.0, section.1.iter().all(|i| self.fields[*i]))
    }

    fn clear_column(&mut self, column: usize) {
        for f in self
            .fields
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| i % (self.width + column) == 0)
            .into_iter()
        {
            *f.1 = false;
        }
    }

    fn clear_row(&mut self, row: usize) {
        let from = row * self.width;
        for i in from..from + self.width {
            self.fields[i] = false;
        }
    }

    fn clear_section(&mut self, section: usize) {
        for i in self.get_section_by_section_index(section) {
            self.fields[i] = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
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
        Board::new_with_fields(width, heigth, section_size, fields);
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
        let board = Board::new_with_fields(6, 9, 3, fields.into());

        board.can_place_piece(x, y, &piece)
    }

    #[test_case(0, true)]
    #[test_case(1, false)]
    fn row_done(row: usize, expected: bool) {
        let board = Board::new_with_fields(2, 2, 2, [true, true, true, false].into());

        assert_eq!(expected, board.row_done(row));
    }

    #[test_case(0, true)]
    #[test_case(1, false)]
    fn column_done(col: usize, expected: bool) {
        let board = Board::new_with_fields(2, 2, 2, [true, true, true, false].into());

        assert_eq!(expected, board.column_done(col));
    }

    #[test_case(0, 0, (0, vec![0, 1, 4, 5]))]
    #[test_case(1, 1, (0, vec![0, 1, 4, 5]))]
    #[test_case(2, 0, (1, vec![2, 3, 6, 7]))]
    #[test_case(0, 2, (2, vec![8, 9, 12, 13]))]
    #[test_case(2, 2, (3, vec![10, 11, 14, 15]))]
    fn get_section(x: usize, y: usize, expected: (usize, Vec<usize>)) {
        let board = Board::new(4, 4, 2);

        assert_eq!(expected, board.get_section(x, y));
    }

    #[test_case(0, vec![0, 1, 4, 5])]
    #[test_case(1, vec![2, 3, 6, 7])]
    #[test_case(2, vec![8, 9, 12, 13])]
    #[test_case(3, vec![10, 11, 14, 15])]
    fn get_section_by_section_index(section_index: usize, expected: Vec<usize>) {
        let board = Board::new(4, 4, 2);

        assert_eq!(expected, board.get_section_by_section_index(section_index));
    }

    #[test_case(0, 0, (0, true))]
    #[test_case(1, 1, (0, true))]
    #[test_case(0, 2, (1, false))]
    fn section_done(x: usize, y: usize, expected: (usize, bool)) {
        let board = Board::new_with_fields(
            2,
            4,
            2,
            [true, true, true, true, true, true, false, false].into(),
        );

        assert_eq!(expected, board.section_done(x, y));
    }
}
